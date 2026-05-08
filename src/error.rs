use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type. Each variant maps to a stable exit code (see
/// [`crate::exit::ExitKind::from_error`]) and a deterministic JSON shape so
/// AI agents can parse failures.
#[derive(Debug, Error)]
pub enum Error {
    #[error("user error: {0}")]
    User(String),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("api error ({status}): {title}")]
    Api {
        status: u16,
        code: Option<String>,
        title: String,
        detail: Option<String>,
        pointer: Option<String>,
        request_id: Option<String>,
        hint: Option<String>,
    },

    #[error("capability not supported on this deployment: {0}")]
    Capability(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    pub fn user(msg: impl Into<String>) -> Self {
        Self::User(msg.into())
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn capability(msg: impl Into<String>) -> Self {
        Self::Capability(msg.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::Serde(value.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Self::Serde(value.to_string())
    }
}
