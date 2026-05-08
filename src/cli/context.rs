use std::sync::Arc;

use crate::{
    config::Profile,
    error::Result,
    output::{resolve_use_color, OutputFormat as OutMode},
};

use super::globals::{GlobalArgs, OutputFormat};

/// Runtime context built from CLI flags + on-disk config. Cheap to clone (Arc).
#[derive(Debug, Clone)]
pub struct Context {
    pub inner: Arc<ContextInner>,
}

#[derive(Debug)]
pub struct ContextInner {
    pub profile: Profile,
    pub format: OutMode,
    pub use_color: bool,
    pub dry_run: bool,
    pub timeout_secs: u64,
    pub retry: u8,
    pub idempotency_key: Option<String>,
}

impl Context {
    pub fn from_globals(globals: &GlobalArgs) -> Result<Self> {
        let profile = Profile::resolve(globals)?;
        let format = if globals.json {
            OutMode::Json
        } else {
            match globals.output {
                Some(OutputFormat::Json) => OutMode::Json,
                Some(OutputFormat::Yaml) => OutMode::Yaml,
                Some(OutputFormat::Tsv) => OutMode::Tsv,
                Some(OutputFormat::Ndjson) => OutMode::Ndjson,
                Some(OutputFormat::Table) | None => OutMode::Table,
            }
        };
        let use_color = resolve_use_color(globals.no_color);
        Ok(Self {
            inner: Arc::new(ContextInner {
                profile,
                format,
                use_color,
                dry_run: globals.dry_run,
                timeout_secs: globals.timeout,
                retry: globals.retry,
                idempotency_key: globals.idempotency_key.clone(),
            }),
        })
    }

    pub fn profile(&self) -> &Profile {
        &self.inner.profile
    }

    pub fn format(&self) -> OutMode {
        self.inner.format
    }

    pub fn use_color(&self) -> bool {
        self.inner.use_color
    }
}
