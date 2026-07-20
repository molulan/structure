use std::fs;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use structure_core::persistence::store::Store;
use tempfile::TempDir;
use tower::ServiceExt;

// The deployed `app` over a throwaway web dir with a stand-in index.html and one
// asset, exercising the static-serving wiring without a real frontend build.
fn app_with_web() -> (TempDir, Router) {
    let dir = tempfile::tempdir().expect("temp dir should create");
    fs::write(dir.path().join("index.html"), "<!doctype html>app-shell")
        .expect("index.html should write");
    fs::create_dir(dir.path().join("assets")).expect("assets dir should create");
    fs::write(dir.path().join("assets").join("app.js"), "console.log(1)")
        .expect("asset should write");

    let store = Store::open(":memory:").expect("in-memory store should open");
    let app = structure_server::app(store, Some(dir.path().to_path_buf()));
    (dir, app)
}

async fn send(app: &Router, method: &str, uri: &str) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

async fn get(app: &Router, uri: &str) -> (StatusCode, String) {
    send(app, "GET", uri).await
}

#[tokio::test]
async fn api_is_served_under_the_api_prefix() {
    let (_dir, app) = app_with_web();

    let (status, body) = get(&app, "/api/mesocycles").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "[]");
}

#[tokio::test]
async fn unknown_api_path_returns_a_json_404_not_the_shell() {
    let (_dir, app) = app_with_web();

    // The bare `/api/` is included because axum's nest matcher doesn't route it
    // inside the API router — it needs the explicit guard in `router`.
    for path in ["/api/does-not-exist", "/api/"] {
        let (status, body) = get(&app, path).await;

        assert_eq!(status, StatusCode::NOT_FOUND, "for {path}");
        assert!(
            body.contains("\"error\""),
            "expected a JSON error body for {path}, got: {body}"
        );
    }
}

#[tokio::test]
async fn bare_api_slash_is_a_json_404_for_any_method() {
    let (_dir, app) = app_with_web();

    let (status, body) = send(&app, "POST", "/api/").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.contains("\"error\""),
        "expected a JSON error body, got: {body}"
    );
}

#[tokio::test]
async fn spa_route_falls_back_to_index_html_rather_than_the_api() {
    let (_dir, app) = app_with_web();

    let (status, body) = get(&app, "/mesocycles/5").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "<!doctype html>app-shell");
}

#[tokio::test]
async fn a_real_asset_is_served_verbatim() {
    let (_dir, app) = app_with_web();

    let (status, body) = get(&app, "/assets/app.js").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "console.log(1)");
}

#[tokio::test]
async fn health_stays_at_the_root() {
    let (_dir, app) = app_with_web();

    let (status, body) = get(&app, "/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}
