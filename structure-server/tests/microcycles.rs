mod common;

use axum::http::StatusCode;
use common::{create_microcycle, create_program, send, test_app};
use serde_json::json;

#[tokio::test]
async fn created_microcycle_appears_in_list() {
    let app = test_app();
    let mesocycle_id = create_program(&app).await;

    let (status, created) = send(
        &app,
        "POST",
        &format!("/mesocycles/{mesocycle_id}/microcycles"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["position"], 0);

    let (status, list) = send(
        &app,
        "GET",
        &format!("/mesocycles/{mesocycle_id}/microcycles"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn list_microcycles_for_missing_program_returns_404() {
    let app = test_app();

    let (status, body) = send(&app, "GET", "/mesocycles/999/microcycles", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn reorder_microcycles_changes_their_order() {
    let app = test_app();
    let mesocycle_id = create_program(&app).await;
    let a = create_microcycle(&app, mesocycle_id).await;
    let b = create_microcycle(&app, mesocycle_id).await;
    let c = create_microcycle(&app, mesocycle_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/mesocycles/{mesocycle_id}/microcycles/order"),
        Some(json!({ "ordered_ids": [c, a, b] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/mesocycles/{mesocycle_id}/microcycles"),
        None,
    )
    .await;
    let ids: Vec<i64> = list
        .as_array()
        .expect("list should be an array")
        .iter()
        .map(|m| m["id"].as_i64().expect("id should be a number"))
        .collect();
    assert_eq!(ids, vec![c, a, b]);
}

#[tokio::test]
async fn reorder_with_mismatching_ids_returns_422() {
    let app = test_app();
    let mesocycle_id = create_program(&app).await;
    let a = create_microcycle(&app, mesocycle_id).await;
    let _b = create_microcycle(&app, mesocycle_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/mesocycles/{mesocycle_id}/microcycles/order"),
        Some(json!({ "ordered_ids": [a] })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_microcycle_removes_it() {
    let app = test_app();
    let mesocycle_id = create_program(&app).await;
    let a = create_microcycle(&app, mesocycle_id).await;
    let _b = create_microcycle(&app, mesocycle_id).await;

    let (status, _) = send(&app, "DELETE", &format!("/microcycles/{a}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/mesocycles/{mesocycle_id}/microcycles"),
        None,
    )
    .await;
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn delete_missing_microcycle_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "DELETE", "/microcycles/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
