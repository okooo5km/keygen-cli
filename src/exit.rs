use std::process::ExitCode;

use crate::error::Error;

/// Stable CLI exit codes. Documented in the AI schema so callers can branch on
/// failure mode without parsing strings.
#[derive(Debug, Clone, Copy)]
pub enum ExitKind {
    Ok = 0,
    UserError = 1,
    ServerError = 2,
    NetworkError = 3,
    AuthError = 4,
    Capability = 5,
}

impl ExitKind {
    pub fn from_error(err: &Error) -> Self {
        match err {
            Error::User(_)
            | Error::Config(_)
            | Error::Serde(_)
            | Error::Io(_)
            | Error::Other(_) => Self::UserError,
            Error::Auth(_) => Self::AuthError,
            Error::Network(_) => Self::NetworkError,
            Error::Capability(_) => Self::Capability,
            Error::Api { status, .. } => {
                if (500..600).contains(status) {
                    Self::ServerError
                } else if *status == 401 || *status == 403 {
                    Self::AuthError
                } else {
                    Self::UserError
                }
            }
        }
    }
}

impl From<ExitKind> for ExitCode {
    fn from(value: ExitKind) -> Self {
        ExitCode::from(value as u8)
    }
}
