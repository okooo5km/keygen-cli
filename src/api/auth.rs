use reqwest::header::{HeaderValue, AUTHORIZATION};

/// Build an `Authorization` header value for the supplied token.
///
/// keygen.sh accepts:
/// - `Bearer <token>` for admin/product/user/environment tokens
/// - `License <key>` for license-key auth
/// - HTTP Basic for the email+password → token mint flow
pub fn bearer(token: &str) -> Option<(reqwest::header::HeaderName, HeaderValue)> {
    HeaderValue::from_str(&format!("Bearer {token}"))
        .ok()
        .map(|v| (AUTHORIZATION, v))
}

pub fn license(key: &str) -> Option<(reqwest::header::HeaderName, HeaderValue)> {
    HeaderValue::from_str(&format!("License {key}"))
        .ok()
        .map(|v| (AUTHORIZATION, v))
}
