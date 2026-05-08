//! Client-side audit for `--filter <relation>=<id>`.
//!
//! Some self-hosted Keygen CE deployments accept the `filter[<relation>]`
//! query param but silently fail to apply it, returning the full collection
//! instead of the filtered subset. The CLI cannot trust the server here, so
//! every `list` runs a post-fetch sanity check: for known relation filter
//! keys, walk the returned resources and confirm each one's
//! `relationships.<key>.data.id` matches the requested id.
//!
//! Authored by okooo5km(十里).

use crate::api::jsonapi::Resource;
use crate::error::{Error, Result};

/// keygen.sh relation filter keys with strong post-condition guarantees:
/// for these, `relationships.<key>.data.id` is expected to equal the
/// filter value, so a mismatch is conclusive evidence the server ignored
/// the filter.
///
/// Attribute filters (`status`, `expires`, `key`, ...) have richer
/// semantics (range, boolean, substring) and are intentionally out of
/// scope here.
pub const RELATION_FILTER_KEYS: &[&str] = &[
    "license",
    "user",
    "product",
    "policy",
    "group",
    "owner",
    "machine",
    "environment",
];

/// Walk every `--filter k=v` whose key is a known relation, and confirm
/// each returned resource carries the expected related-id. A single
/// mismatch returns `Error::Api { code: "FILTER_UNSUPPORTED", ... }`.
///
/// An empty collection is treated as a valid filtered result — the
/// server may have legitimately returned nothing.
pub fn audit(filters: &[String], resources: &[Resource]) -> Result<()> {
    for entry in filters {
        let Some((key, expected)) = entry.split_once('=') else {
            continue;
        };
        if !RELATION_FILTER_KEYS.contains(&key) {
            continue;
        }
        for r in resources {
            let actual = r
                .relationships
                .as_ref()
                .and_then(|rels| rels.get(key))
                .and_then(|rel| rel.get("data"))
                .and_then(|d| d.get("id"))
                .and_then(|v| v.as_str());
            if actual != Some(expected) {
                return Err(Error::Api {
                    status: 200,
                    code: Some("FILTER_UNSUPPORTED".into()),
                    title: format!(
                        "server ignored --filter {key}={expected}; \
                         got results that do not match the filter"
                    ),
                    detail: Some(
                        "the active deployment (likely self-hosted CE) accepted the \
                         filter[<relation>] query param but did not apply it. \
                         The CLI detected this by scanning the returned collection."
                            .into(),
                    ),
                    pointer: None,
                    request_id: None,
                    hint: Some(
                        "run `keygen doctor` to see whether this deployment supports \
                         relation filters; if not, list without the filter and \
                         narrow the result set client-side."
                            .into(),
                    ),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn res_with_rel(id: &str, rel_key: &str, rel_id: &str) -> Resource {
        Resource {
            id: id.into(),
            r#type: "machines".into(),
            attributes: json!({}),
            relationships: Some(json!({
                rel_key: { "data": { "type": "licenses", "id": rel_id } }
            })),
        }
    }

    #[test]
    fn passes_when_all_match() {
        let filters = vec!["license=lic_abc".to_string()];
        let resources = vec![
            res_with_rel("m1", "license", "lic_abc"),
            res_with_rel("m2", "license", "lic_abc"),
        ];
        assert!(audit(&filters, &resources).is_ok());
    }

    #[test]
    fn passes_on_empty_collection() {
        let filters = vec!["license=lic_abc".to_string()];
        assert!(audit(&filters, &[]).is_ok());
    }

    #[test]
    fn passes_when_filter_key_is_not_a_known_relation() {
        let filters = vec!["status=ACTIVE".to_string()];
        let resources = vec![res_with_rel("m1", "license", "lic_xyz")];
        assert!(audit(&filters, &resources).is_ok());
    }

    #[test]
    fn flags_mismatch_as_filter_unsupported() {
        let filters = vec!["license=lic_abc".to_string()];
        let resources = vec![res_with_rel("m1", "license", "lic_other")];
        let err = audit(&filters, &resources).expect_err("mismatch should error");
        match err {
            Error::Api { code, .. } => {
                assert_eq!(code.as_deref(), Some("FILTER_UNSUPPORTED"));
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    #[test]
    fn flags_missing_relationship_as_filter_unsupported() {
        let filters = vec!["license=lic_abc".to_string()];
        let resources = vec![Resource {
            id: "m1".into(),
            r#type: "machines".into(),
            attributes: json!({}),
            relationships: None,
        }];
        let err = audit(&filters, &resources).expect_err("missing relation should error");
        match err {
            Error::Api { code, .. } => assert_eq!(code.as_deref(), Some("FILTER_UNSUPPORTED")),
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }
}
