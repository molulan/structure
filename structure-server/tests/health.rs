use axum::body::Body;
use axum::http::{Request, StatusCode};
use structure_core::persistence::store::Store;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let store = Store::open(":memory:").expect("in-memory store should open");
    let app = structure_server::router(store);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    assert_eq!(&body[..], b"ok");
}
