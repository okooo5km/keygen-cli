use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use url::Url;

use crate::{cli::Context, config::Profile, error::Result};

const JSONAPI_TYPE: &str = "application/vnd.api+json";

/// Thin wrapper around `reqwest::Client` that injects JSON:API headers, the
/// configured Authorization header, and computes account-scoped URLs.
#[derive(Debug, Clone)]
pub struct Client {
    inner: reqwest::Client,
    base: Url,
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

        let inner = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(ctx.inner.timeout_secs))
            .build()?;

        let base = base_url(ctx.profile())?;
        Ok(Self { inner, base })
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.inner
    }

    pub fn base(&self) -> &Url {
        &self.base
    }
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
            .map_err(|()| crate::Error::config("host URL cannot be a base"))?;
        segments.pop_if_empty();
        segments.push("v1");
        if let AccountMode::Multiplayer = profile.mode {
            let account = profile.account.as_deref().ok_or_else(|| {
                crate::Error::config(
                    "multiplayer/Official deployment requires --account or KEYGEN_ACCOUNT",
                )
            })?;
            segments.push("accounts");
            segments.push(account);
        }
    }
    Ok(url)
}
