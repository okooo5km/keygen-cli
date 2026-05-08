use serde_json::json;

use crate::{
    api::{client::Query, Client},
    capability,
    cli::Context,
    error::{Error, Result},
};

pub async fn run(ctx: &Context) -> Result<()> {
    let mut report = json!({
        "ok": true,
        "data": {
            "profile": ctx.profile().name,
            "deployment": format!("{:?}", ctx.profile().deployment).to_lowercase(),
            "host": ctx.profile().host.as_str(),
            "account": ctx.profile().account,
            "checks": [],
        }
    });
    let mut all_ok = true;
    let mut checks: Vec<serde_json::Value> = Vec::new();

    // 1. Build an HTTP client (validates host + token resolution).
    let client = match Client::new(ctx) {
        Ok(c) => {
            checks.push(check("client", true, "client built"));
            Some(c)
        }
        Err(e) => {
            all_ok = false;
            checks.push(check("client", false, &e.to_string()));
            None
        }
    };

    // 2. Reach /v1/ping (or /v1/profile as a fallback).
    if let Some(ref c) = client {
        match c
            .get::<crate::api::jsonapi::Resource>("/profile", &Query::new())
            .await
        {
            Ok(_) => checks.push(check("auth", true, "/profile returned 2xx")),
            Err(Error::Api { status, .. }) if status == 401 || status == 403 => {
                all_ok = false;
                checks.push(check("auth", false, "token rejected — run `keygen login`"));
            }
            Err(e) => {
                all_ok = false;
                checks.push(check("auth", false, &e.to_string()));
            }
        }
    }

    // 3. Capability probe.
    let caps = capability::detect::refresh(ctx).await.unwrap_or_default();
    checks.push(json!({
        "name": "capabilities",
        "ok": true,
        "detail": {
            "environments": caps.environments,
            "event_logs": caps.event_logs,
            "request_logs": caps.request_logs,
            "sso": caps.sso,
            "oci_registry": caps.oci_registry,
            "import_export": caps.import_export,
        }
    }));

    report["ok"] = json!(all_ok);
    report["data"]["checks"] = json!(checks);
    crate::output::json::print(&report)?;
    if all_ok {
        Ok(())
    } else {
        Err(Error::user("doctor: one or more checks failed"))
    }
}

fn check(name: &str, ok: bool, detail: &str) -> serde_json::Value {
    json!({ "name": name, "ok": ok, "detail": detail })
}
