//! Shared building blocks for resource subcommands.
//!
//! Every resource follows the same CRUD shape:
//!
//! ```text
//! keygen <resource> list      [--filter k=v ...] [--limit N] [--page N] [--include rel] [--sort field]
//! keygen <resource> get       <id> [--include rel]
//! keygen <resource> create    [--from-file <json>|-] [--<attr> ...] [--metadata k=v ...]
//! keygen <resource> update    <id> [--from-file <json>|-] [--<attr> ...]
//! keygen <resource> delete    <id> [--yes]
//! ```
//!
//! Resource-specific actions (validate / suspend / publish / ...) are added to
//! the per-resource `Cmd` enum on top of these.

use std::io::Read;

use clap::Args;
use serde_json::{json, Map, Value};

use crate::{
    api::{client::Query, jsonapi::Resource, Client},
    cli::Context,
    error::{Error, Result},
};

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Filter rows. May be passed multiple times: `--filter status=ACTIVE`.
    #[arg(long, value_name = "K=V")]
    pub filter: Vec<String>,

    /// Page number (1-based).
    #[arg(long, default_value_t = 1)]
    pub page: u64,

    /// Page size.
    #[arg(long, default_value_t = 50)]
    pub limit: u64,

    /// Sort field (prefix with `-` for descending).
    #[arg(long)]
    pub sort: Option<String>,

    /// Include related resources, comma separated.
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GetArgs {
    /// Resource id.
    pub id: String,

    /// Include related resources, comma separated.
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct DeleteArgs {
    /// Resource id.
    pub id: String,

    /// Skip the confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CreateArgs {
    /// Read the full JSON body from a file (or `-` for stdin).
    #[arg(long, value_name = "PATH|-")]
    pub from_file: Option<String>,

    /// Metadata entries `k=v`. May be repeated.
    #[arg(long, value_name = "K=V")]
    pub metadata: Vec<String>,

    /// JSONPath-style attribute overrides, e.g. `--set attrs.maxMachines=5`.
    #[arg(long, value_name = "PATH=VALUE")]
    pub set: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    /// Resource id.
    pub id: String,

    /// Read the full JSON body from a file (or `-` for stdin).
    #[arg(long, value_name = "PATH|-")]
    pub from_file: Option<String>,

    /// Metadata entries `k=v`. May be repeated.
    #[arg(long, value_name = "K=V")]
    pub metadata: Vec<String>,

    /// JSONPath-style attribute overrides.
    #[arg(long, value_name = "PATH=VALUE")]
    pub set: Vec<String>,
}

/// Resource description: the JSON:API `type` and the URL stem.
pub struct Crud {
    pub jsonapi_type: &'static str,
    pub path: &'static str,
}

impl Crud {
    pub const fn new(jsonapi_type: &'static str, path: &'static str) -> Self {
        Self { jsonapi_type, path }
    }

    pub async fn list(&self, ctx: &Context, args: &ListArgs) -> Result<Vec<Resource>> {
        let client = Client::new(ctx)?;
        let query = Query::new()
            .page(args.page, args.limit)
            .filters(args.filter.clone())
            .include(args.include.clone())
            .sort(args.sort.clone());
        let doc = client.get::<Vec<Resource>>(self.path, &query).await?;
        crate::api::filter_audit::audit(&args.filter, &doc.data)?;
        Ok(doc.data)
    }

    pub async fn get(&self, ctx: &Context, args: &GetArgs) -> Result<Resource> {
        let client = Client::new(ctx)?;
        let query = Query::new().include(args.include.clone());
        let path = format!("{}/{}", self.path, args.id);
        let doc = client.get::<Resource>(&path, &query).await?;
        Ok(doc.data)
    }

    pub async fn create(&self, ctx: &Context, args: &CreateArgs) -> Result<Resource> {
        let body = self.build_body(args.from_file.as_deref(), &args.metadata, &args.set, None)?;
        let client = Client::new(ctx)?;
        let doc = client.post::<_, Resource>(self.path, &body).await?;
        Ok(doc.data)
    }

    pub async fn update(&self, ctx: &Context, args: &UpdateArgs) -> Result<Resource> {
        let body = self.build_body(
            args.from_file.as_deref(),
            &args.metadata,
            &args.set,
            Some(&args.id),
        )?;
        let client = Client::new(ctx)?;
        let path = format!("{}/{}", self.path, args.id);
        let doc = client.patch::<_, Resource>(&path, &body).await?;
        Ok(doc.data)
    }

    pub async fn delete(&self, ctx: &Context, args: &DeleteArgs) -> Result<()> {
        if !args.yes && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            let prompt = format!(
                "Delete {ty} {id}? Type the id to confirm: ",
                ty = self.jsonapi_type,
                id = args.id
            );
            let typed = inquire::Text::new(&prompt)
                .prompt()
                .map_err(|e| Error::user(format!("aborted: {e}")))?;
            if typed != args.id {
                return Err(Error::user("delete cancelled (id did not match)"));
            }
        }
        let client = Client::new(ctx)?;
        let path = format!("{}/{}", self.path, args.id);
        client.delete(&path).await
    }

    /// Construct a JSON:API request body for create / update.
    ///
    /// Order of operations:
    /// 1. If `--from-file` is given, parse it as the *full* `{ data: {...} }`
    ///    document (so AI clients can hand a complete object). When the parsed
    ///    JSON looks like just `attributes` we wrap it.
    /// 2. Apply `--set attrs.foo=bar` JSON-path overrides.
    /// 3. Merge `--metadata k=v` into `attributes.metadata`.
    /// 4. Ensure `data.type` matches the resource (and `data.id` for updates).
    fn build_body(
        &self,
        from_file: Option<&str>,
        metadata: &[String],
        set: &[String],
        update_id: Option<&str>,
    ) -> Result<Value> {
        let mut doc = match from_file {
            Some("-") => parse_json_from_stdin()?,
            Some(path) => parse_json_from_path(path)?,
            None => json!({ "data": { "type": self.jsonapi_type, "attributes": {} } }),
        };
        normalize_doc_shape(&mut doc, self.jsonapi_type);

        // Stamp type / id and merge --metadata into attributes.metadata before
        // applying --set so user overrides win.
        if let Some(data) = doc.get_mut("data").and_then(Value::as_object_mut) {
            data.insert("type".into(), json!(self.jsonapi_type));
            if let Some(id) = update_id {
                data.insert("id".into(), json!(id));
            }
            let attrs = data
                .entry("attributes")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .ok_or_else(|| Error::user("data.attributes must be an object"))?;
            apply_metadata(attrs, metadata)?;
        } else {
            return Err(Error::user("request body must contain a `data` object"));
        }

        apply_set_overrides_doc(&mut doc, set)?;

        Ok(doc)
    }
}

fn parse_json_from_path(path: &str) -> Result<Value> {
    let raw = std::fs::read(path)?;
    serde_json::from_slice(&raw).map_err(Error::from)
}

fn parse_json_from_stdin() -> Result<Value> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    serde_json::from_slice(&buf).map_err(Error::from)
}

/// Accept three input shapes from `--from-file`:
/// - `{ "data": { ... } }` (canonical JSON:API document)
/// - `{ "type": ..., "attributes": ... }` (resource object — wrap it)
/// - `{ "name": ..., "maxMachines": ... }` (loose attributes — wrap it)
fn normalize_doc_shape(doc: &mut Value, jsonapi_type: &str) {
    if !doc.is_object() {
        return;
    }
    if doc.get("data").is_some() {
        return;
    }
    let inner = std::mem::take(doc);
    if inner.get("type").is_some() && inner.get("attributes").is_some() {
        *doc = json!({ "data": inner });
    } else {
        *doc = json!({
            "data": {
                "type": jsonapi_type,
                "attributes": inner,
            }
        });
    }
}

/// Apply `--set PATH=VALUE` overrides against the *full* JSON:API document.
///
/// Path conventions:
/// - `data.<...>`         — absolute, from the document root
/// - `attrs.<...>`        — shorthand for `data.attributes.<...>`
/// - `attributes.<...>`   — shorthand for `data.attributes.<...>`
/// - anything else        — treated as `data.attributes.<...>` (back-compat)
///
/// `VALUE` is parsed as JSON first; any parse failure falls back to a string.
fn apply_set_overrides_doc(doc: &mut Value, set: &[String]) -> Result<()> {
    for entry in set {
        let (raw_path, raw_value) = entry
            .split_once('=')
            .ok_or_else(|| Error::user(format!("--set expects PATH=VALUE, got `{entry}`")))?;
        let resolved = if let Some(rest) = raw_path.strip_prefix("data.") {
            format!("data.{rest}")
        } else if let Some(rest) = raw_path.strip_prefix("attrs.") {
            format!("data.attributes.{rest}")
        } else if let Some(rest) = raw_path.strip_prefix("attributes.") {
            format!("data.attributes.{rest}")
        } else {
            format!("data.attributes.{raw_path}")
        };
        let value: Value = serde_json::from_str(raw_value).unwrap_or_else(|_| json!(raw_value));
        let root = doc
            .as_object_mut()
            .ok_or_else(|| Error::user("request body must be a JSON object"))?;
        set_pointer(root, &resolved, value);
    }
    Ok(())
}

fn apply_metadata(attrs: &mut Map<String, Value>, metadata: &[String]) -> Result<()> {
    if metadata.is_empty() {
        return Ok(());
    }
    let entry = attrs
        .entry("metadata")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| Error::user("attributes.metadata must be an object"))?;
    for raw in metadata {
        let (k, v) = raw
            .split_once('=')
            .ok_or_else(|| Error::user(format!("--metadata expects K=V, got `{raw}`")))?;
        let value: Value = serde_json::from_str(v).unwrap_or_else(|_| json!(v));
        entry.insert(k.into(), value);
    }
    Ok(())
}

/// Set a dotted path on a JSON object, creating intermediate objects as needed.
fn set_pointer(root: &mut Map<String, Value>, path: &str, value: Value) {
    let mut parts = path.split('.').peekable();
    let mut current = root;
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current.insert(part.to_string(), value);
            return;
        }
        let entry = current.entry(part.to_string()).or_insert_with(|| json!({}));
        if !entry.is_object() {
            *entry = json!({});
        }
        current = entry.as_object_mut().expect("just ensured object");
    }
}
