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
        no_color: true,
        quiet: false,
        verbose: 0,
        ai: true,
        human: false,
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
