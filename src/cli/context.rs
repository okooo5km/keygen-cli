use std::sync::Arc;

use crate::{
    config::Profile,
    error::Result,
    output::{Mode, ModeInputs, OutputFormat as OutMode},
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
    pub mode: Mode,
    pub format: OutMode,
    pub dry_run: bool,
    pub timeout_secs: u64,
    pub retry: u8,
    pub idempotency_key: Option<String>,
}

impl Context {
    pub fn from_globals(globals: &GlobalArgs) -> Result<Self> {
        let profile = Profile::resolve(globals)?;
        let mode = Mode::resolve(ModeInputs {
            ai_flag: globals.ai,
            human_flag: globals.human,
            no_color: globals.no_color,
        });
        let format = match globals.output {
            Some(OutputFormat::Table) => OutMode::Table,
            Some(OutputFormat::Json) => OutMode::Json,
            Some(OutputFormat::Yaml) => OutMode::Yaml,
            Some(OutputFormat::Tsv) => OutMode::Tsv,
            Some(OutputFormat::Ndjson) => OutMode::Ndjson,
            None => mode.default_format(),
        };
        Ok(Self {
            inner: Arc::new(ContextInner {
                profile,
                mode,
                format,
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

    pub fn mode(&self) -> Mode {
        self.inner.mode
    }

    pub fn format(&self) -> OutMode {
        self.inner.format
    }
}
