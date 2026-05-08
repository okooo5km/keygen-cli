use clap::Args;
use serde_json::json;

use crate::{
    api::{client::Query, Client},
    capability,
    cli::Context,
    config::file,
    error::{Error, Result},
};

#[derive(Debug, Clone, Args)]
pub struct DoctorArgs {
    /// Force a fresh capability probe and clear the on-disk cache.
    #[arg(long)]
    pub refresh: bool,
}

pub async fn run(ctx: &Context, args: DoctorArgs) -> Result<()> {
    if args.refresh {
        clear_cache_silently();
    }

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
    if args.refresh {
        report["data"]["refreshed"] = json!(true);
    }
    crate::output::json::print(&report)?;
    if all_ok {
        Ok(())
    } else {
        Err(Error::user("doctor: one or more checks failed"))
    }
}

fn clear_cache_silently() {
    if let Ok(dir) = file::cache_dir() {
        let path = dir.join("capabilities.json");
        let _ = std::fs::remove_file(path);
    }
}

fn check(name: &str, ok: bool, detail: &str) -> serde_json::Value {
    json!({ "name": name, "ok": ok, "detail": detail })
}
