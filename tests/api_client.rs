//! Smoke tests for `keygen_cli::api::Client`.
//!
//! Stand up a wiremock server, point a Profile at it, and verify GET happy-path
//! plus 404 → `Error::Api` mapping.

use keygen_cli::{
    api::{
        client::{Client, Query},
        jsonapi::Resource,
    },
    cli::{globals::GlobalArgs, Context},
    resources::common::{Crud, ListArgs},
    Error,
};
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

fn globals(host: &str) -> GlobalArgs {
    GlobalArgs {
        profile: Some("test".into()),
        host: Some(host.to_string()),
        account: None,
        token: Some("kgn_test_xyz".into()),
        env: None,
        output: Some(keygen_cli::cli::globals::OutputFormat::Json),
        json: false,
        layout: None,
        cards: false,
        no_color: true,
        quiet: false,
        verbose: 0,
        dry_run: false,
        idempotency_key: None,
        timeout: 5,
        retry: 0,
    }
}

#[tokio::test]
async fn get_returns_typed_document() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/licenses"))
        .and(query_param("page[number]", "1"))
        .and(header("authorization", "Bearer kgn_test_xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":[{"id":"abc","type":"licenses","attributes":{"name":"demo","status":"ACTIVE"}}]}"#,
            "application/vnd.api+json",
        ))
        .mount(&server)
        .await;

    let ctx = Context::from_globals(&globals(&server.uri())).expect("ctx");
    let client = Client::new(&ctx).expect("client");
    let doc = client
        .get::<Vec<Resource>>("/licenses", &Query::new().page(1, 10))
        .await
        .expect("ok response");

    assert_eq!(doc.data.len(), 1);
    assert_eq!(doc.data[0].id, "abc");
    assert_eq!(doc.data[0].r#type, "licenses");
}

#[tokio::test]
async fn http_404_is_mapped_to_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/licenses/missing"))
        .respond_with(
            ResponseTemplate::new(404)
                .insert_header("x-request-id", "req-123")
                .set_body_raw(
                    r#"{"errors":[{"title":"Not found","detail":"License not found","code":"LICENSE_NOT_FOUND","source":{"pointer":"/data"}}]}"#,
                    "application/vnd.api+json",
                ),
        )
        .mount(&server)
        .await;

    let ctx = Context::from_globals(&globals(&server.uri())).expect("ctx");
    let client = Client::new(&ctx).expect("client");
    let err = client
        .get::<Resource>("/licenses/missing", &Query::new())
        .await
        .expect_err("404 should error");

    match err {
        Error::Api {
            status,
            code,
            request_id,
            ..
        } => {
            assert_eq!(status, 404);
            assert_eq!(code.as_deref(), Some("LICENSE_NOT_FOUND"));
            assert_eq!(request_id.as_deref(), Some("req-123"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

fn list_args_with_filter(filter: &str) -> ListArgs {
    ListArgs {
        filter: vec![filter.into()],
        page: 1,
        limit: 10,
        sort: None,
        include: vec![],
    }
}

#[tokio::test]
async fn list_passes_audit_when_relations_match() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/machines"))
        .and(query_param("license", "lic_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":[{
                "id":"m1","type":"machines","attributes":{},
                "relationships":{"license":{"data":{"type":"licenses","id":"lic_abc"}}}
            }]}"#,
            "application/vnd.api+json",
        ))
        .mount(&server)
        .await;

    let ctx = Context::from_globals(&globals(&server.uri())).expect("ctx");
    let crud = Crud::new("machines", "/machines");
    let rows = crud
        .list(&ctx, &list_args_with_filter("license=lic_abc"))
        .await
        .expect("audit should pass");
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn list_flags_filter_unsupported_when_server_lies() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/machines"))
        .and(query_param("license", "lic_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":[{
                "id":"m1","type":"machines","attributes":{},
                "relationships":{"license":{"data":{"type":"licenses","id":"lic_other"}}}
            }]}"#,
            "application/vnd.api+json",
        ))
        .mount(&server)
        .await;

    let ctx = Context::from_globals(&globals(&server.uri())).expect("ctx");
    let crud = Crud::new("machines", "/machines");
    let err = crud
        .list(&ctx, &list_args_with_filter("license=lic_abc"))
        .await
        .expect_err("audit should detect server-ignored filter");

    match err {
        Error::Api { code, .. } => {
            assert_eq!(code.as_deref(), Some("FILTER_UNSUPPORTED"));
        }
        other => panic!("expected FILTER_UNSUPPORTED, got {other:?}"),
    }
}

#[tokio::test]
async fn doctor_probe_detects_ignored_relation_filter() {
    use keygen_cli::capability::detect;

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":{"id":"u1","type":"users","attributes":{}}}"#,
            "application/vnd.api+json",
        ))
        .mount(&server)
        .await;

    // CE: no /environments — return 404 so the EE branch stays off.
    Mock::given(method("GET"))
        .and(path("/v1/environments"))
        .respond_with(ResponseTemplate::new(404).set_body_raw(
            r#"{"errors":[{"title":"Not found"}]}"#,
            "application/vnd.api+json",
        ))
        .mount(&server)
        .await;

    // CE-style: server ignores filter[license] and returns a non-empty
    // collection even for a never-matching license id.
    Mock::given(method("GET"))
        .and(path("/v1/machines"))
        .and(query_param(
            "license",
            "00000000-0000-0000-0000-000000000000",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":[{"id":"m1","type":"machines","attributes":{}}]}"#,
            "application/vnd.api+json",
        ))
        .mount(&server)
        .await;

    let mut g = globals(&server.uri());
    // Force a CE-shaped probe (singleplayer) — but globals() leaves account
    // unset which is fine; deployment defaults to whatever Profile picks.
    g.profile = Some("ce-probe".into());
    let ctx = Context::from_globals(&g).expect("ctx");
    let caps = detect::refresh(&ctx).await.expect("refresh");
    assert_eq!(caps.filters_relation, Some(false));
}
