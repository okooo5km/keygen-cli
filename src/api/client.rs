use std::time::Duration;

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    Method, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tracing::{debug, info};
use url::Url;

use crate::{
    api::{auth as auth_header, error::map_error, jsonapi::Document},
    auth::store,
    cli::Context,
    config::Profile,
    error::{Error, Result},
};

const JSONAPI_TYPE: &str = "application/vnd.api+json";
const REQUEST_ID_HEADER: &str = "x-request-id";
const IDEMPOTENCY_HEADER: &str = "idempotency-key";

/// Thin wrapper around `reqwest::Client` that injects JSON:API headers, the
/// configured `Authorization` header, and computes account-scoped URLs.
///
/// One client per command run. `--dry-run` short-circuits before the network
/// call; `--verbose` (or `KEYGEN_LOG=debug`) emits request/response traces on
/// stderr via `tracing`.
#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    base: Url,
    auth: Option<HeaderValue>,
    dry_run: bool,
    idempotency_key: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Query {
    pairs: Vec<(String, String)>,
}

#[allow(clippy::return_self_not_must_use)]
impl Query {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pair(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.pairs.push((k.into(), v.into()));
        self
    }

    pub fn page(mut self, number: u64, size: u64) -> Self {
        self.pairs.push(("page[number]".into(), number.to_string()));
        self.pairs.push(("page[size]".into(), size.to_string()));
        self
    }

    /// `filter[k]=v`. Accepts `k=v` strings (skips malformed entries).
    pub fn filters<I: IntoIterator<Item = String>>(mut self, raw: I) -> Self {
        for entry in raw {
            if let Some((k, v)) = entry.split_once('=') {
                self.pairs.push((format!("filter[{k}]"), v.to_string()));
            }
        }
        self
    }

    pub fn include<I: IntoIterator<Item = String>>(mut self, rels: I) -> Self {
        let joined: Vec<_> = rels.into_iter().collect();
        if !joined.is_empty() {
            self.pairs.push(("include".into(), joined.join(",")));
        }
        self
    }

    pub fn sort(mut self, field: Option<String>) -> Self {
        if let Some(f) = field {
            self.pairs.push(("sort".into(), f));
        }
        self
    }
}

impl Client {
    pub fn new(ctx: &Context) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static(JSONAPI_TYPE));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(JSONAPI_TYPE));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(concat!("keygen-cli/", env!("CARGO_PKG_VERSION"))),
        );
        if let Some(env_id) = ctx.profile().env.as_deref() {
            if let Ok(v) = HeaderValue::from_str(env_id) {
                headers.insert(HeaderName::from_static("keygen-environment"), v);
            }
        }

        let inner = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(ctx.inner.timeout_secs))
            .build()?;

        let base = base_url(ctx.profile())?;
        let auth = resolve_auth(ctx.profile())?;

        Ok(Self {
            inner,
            base,
            auth,
            dry_run: ctx.inner.dry_run,
            idempotency_key: ctx.inner.idempotency_key.clone(),
        })
    }

    pub fn base(&self) -> &Url {
        &self.base
    }

    /// `GET <path>` returning a typed JSON:API document.
    pub async fn get<T: DeserializeOwned>(&self, path: &str, query: &Query) -> Result<Document<T>> {
        self.send_typed(Method::GET, path, query, Option::<&Value>::None)
            .await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Document<T>> {
        self.send_typed(Method::POST, path, &Query::new(), Some(body))
            .await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<Document<T>> {
        self.send_typed(Method::PATCH, path, &Query::new(), Some(body))
            .await
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let resp = self
            .send_raw(Method::DELETE, path, &Query::new(), Option::<&Value>::None)
            .await?;
        if let Some(resp) = resp {
            handle_no_body(resp).await?;
        }
        Ok(())
    }

    /// Lower-level send returning the parsed [`Document<T>`]. Honours
    /// `--dry-run` by printing the request and returning a fabricated empty
    /// document (only the matching variants will succeed).
    async fn send_typed<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: &Query,
        body: Option<&B>,
    ) -> Result<Document<T>> {
        let resp = self.send_raw(method, path, query, body).await?;
        match resp {
            Some(r) => parse_response(r).await,
            None => Err(Error::user(
                "dry-run: no response (use --output json to inspect the request envelope)",
            )),
        }
    }

    async fn send_raw<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        query: &Query,
        body: Option<&B>,
    ) -> Result<Option<Response>> {
        let url = self.url_for(path, query)?;
        if self.dry_run {
            print_dry_run(&method, &url, body)?;
            return Ok(None);
        }

        let mut req = self.inner.request(method.clone(), url.clone());
        if let Some(auth) = &self.auth {
            req = req.header(AUTHORIZATION, auth.clone());
        }
        if matches!(
            method,
            Method::POST | Method::PATCH | Method::PUT | Method::DELETE
        ) {
            if let Some(key) = &self.idempotency_key {
                req = req.header(IDEMPOTENCY_HEADER, key);
            }
        }
        if let Some(b) = body {
            req = req.json(b);
        }

        debug!(method = %method, url = %url, "→ keygen request");
        let resp = req.send().await?;
        info!(method = %method, url = %url, status = resp.status().as_u16(), "← keygen response");
        Ok(Some(resp))
    }

    fn url_for(&self, path: &str, query: &Query) -> Result<Url> {
        let mut url = self.base.clone();
        {
            let mut seg = url
                .path_segments_mut()
                .map_err(|()| Error::config("base URL cannot be a base"))?;
            for part in path
                .trim_start_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
            {
                seg.push(part);
            }
        }
        if !query.pairs.is_empty() {
            let mut qp = url.query_pairs_mut();
            for (k, v) in &query.pairs {
                qp.append_pair(k, v);
            }
            drop(qp);
        }
        Ok(url)
    }
}

async fn parse_response<T: DeserializeOwned>(resp: Response) -> Result<Document<T>> {
    let status = resp.status();
    let request_id = resp
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp.bytes().await?;

    if status.is_success() {
        if status == StatusCode::NO_CONTENT || bytes.is_empty() {
            return Err(Error::user(
                "expected JSON:API document but got empty body — use the no-body variant",
            ));
        }
        serde_json::from_slice::<Document<T>>(&bytes).map_err(Error::from)
    } else {
        Err(map_error(status.as_u16(), request_id, &bytes))
    }
}

async fn handle_no_body(resp: Response) -> Result<()> {
    let status = resp.status();
    let request_id = resp
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    if status.is_success() {
        return Ok(());
    }
    let bytes = resp.bytes().await?;
    Err(map_error(status.as_u16(), request_id, &bytes))
}

fn resolve_auth(profile: &Profile) -> Result<Option<HeaderValue>> {
    if let Some(token) = &profile.token_override {
        return Ok(auth_header::bearer(token).map(|(_, v)| v));
    }
    if let Some(token) = store::load_token(&profile.name)? {
        return Ok(auth_header::bearer(&token).map(|(_, v)| v));
    }
    Ok(None)
}

/// Compute the JSON:API base URL given the profile.
///
/// - Official: `https://api.keygen.sh/v1/accounts/<account>`
/// - Self-hosted multiplayer: `https://<host>/v1/accounts/<account>`
/// - Self-hosted singleplayer: `https://<host>/v1`
fn base_url(profile: &Profile) -> Result<Url> {
    use crate::config::profile::AccountMode;

    let mut url = profile.host.clone();
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| Error::config("host URL cannot be a base"))?;
        segments.pop_if_empty();
        segments.push("v1");
        if let AccountMode::Multiplayer = profile.mode {
            let account = profile.account.as_deref().ok_or_else(|| {
                Error::config(
                    "multiplayer/Official deployment requires --account or KEYGEN_ACCOUNT",
                )
            })?;
            segments.push("accounts");
            segments.push(account);
        }
    }
    Ok(url)
}

fn print_dry_run<B: Serialize>(method: &Method, url: &Url, body: Option<&B>) -> Result<()> {
    let body_json = match body {
        Some(b) => Some(serde_json::to_value(b)?),
        None => None,
    };
    let envelope = serde_json::json!({
        "ok": true,
        "data": {
            "dry_run": true,
            "method": method.as_str(),
            "url": url.as_str(),
            "body": body_json,
        }
    });
    println!("{}", serde_json::to_string(&envelope)?);
    Ok(())
}
