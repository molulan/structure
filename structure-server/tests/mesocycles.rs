mod common;

use axum::http::StatusCode;
use common::{send, test_app};
use serde_json::json;

#[tokio::test]
async fn create_then_get_returns_the_mesocycle() {
    let app = test_app();

    let (status, created) = send(
        &app,
        "POST",
        "/mesocycles",
        Some(json!({ "name": "Hypertrophy", "mode": "Manual" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Hypertrophy");
    assert_eq!(created["mode"], "Manual");
    let id = created["id"].as_i64().expect("id should be a number");

    let (status, fetched) = send(&app, "GET", &format!("/mesocycles/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"].as_i64(), Some(id));
    assert_eq!(fetched["name"], "Hypertrophy");
    assert_eq!(fetched["microcycle_count"], 0);
}

#[tokio::test]
async fn list_returns_created_mesocycles() {
    let app = test_app();
    send(
        &app,
        "POST",
        "/mesocycles",
        Some(json!({ "name": "A", "mode": "Manual" })),
    )
    .await;
    send(
        &app,
        "POST",
        "/mesocycles",
        Some(json!({ "name": "B", "mode": "Algorithmic" })),
    )
    .await;

    let (status, list) = send(&app, "GET", "/mesocycles", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 2);
}

#[tokio::test]
async fn get_missing_mesocycle_returns_404() {
    let app = test_app();

    let (status, body) = send(&app, "GET", "/mesocycles/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn update_changes_name_and_mode() {
    let app = test_app();
    let (_, created) = send(
        &app,
        "POST",
        "/mesocycles",
        Some(json!({ "name": "Old", "mode": "Manual" })),
    )
    .await;
    let id = created["id"].as_i64().expect("id should be a number");

    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/mesocycles/{id}"),
        Some(json!({ "name": "New", "mode": "Algorithmic" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "New");
    assert_eq!(updated["mode"], "Algorithmic");
}

#[tokio::test]
async fn delete_then_get_returns_404() {
    let app = test_app();
    let (_, created) = send(
        &app,
        "POST",
        "/mesocycles",
        Some(json!({ "name": "Temp", "mode": "Manual" })),
    )
    .await;
    let id = created["id"].as_i64().expect("id should be a number");

    let (status, _) = send(&app, "DELETE", &format!("/mesocycles/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send(&app, "GET", &format!("/mesocycles/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_missing_mesocycle_returns_404() {
    let app = test_app();

    let (status, _) = send(
        &app,
        "PUT",
        "/mesocycles/999",
        Some(json!({ "name": "X", "mode": "Manual" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
