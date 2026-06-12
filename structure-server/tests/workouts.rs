mod common;

use axum::http::StatusCode;
use common::{create_microcycle, create_program, create_workout, send, test_app};
use serde_json::json;

async fn microcycle(app: &axum::Router) -> i64 {
    let mesocycle_id = create_program(app).await;
    create_microcycle(app, mesocycle_id).await
}

#[tokio::test]
async fn created_workout_appears_in_list() {
    let app = test_app();
    let microcycle_id = microcycle(&app).await;

    let (status, created) = send(
        &app,
        "POST",
        &format!("/microcycles/{microcycle_id}/workouts"),
        Some(json!({ "name": "Push" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Push");
    assert_eq!(created["position"], 0);

    let (status, list) = send(
        &app,
        "GET",
        &format!("/microcycles/{microcycle_id}/workouts"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn list_workouts_for_missing_microcycle_returns_404() {
    let app = test_app();

    let (status, body) = send(&app, "GET", "/microcycles/999/workouts", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn rename_workout_changes_its_name() {
    let app = test_app();
    let microcycle_id = microcycle(&app).await;
    let id = create_workout(&app, microcycle_id, "Push").await;

    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/workouts/{id}"),
        Some(json!({ "name": "Upper" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "Upper");
    assert_eq!(updated["position"], 0);
}

#[tokio::test]
async fn rename_missing_workout_returns_404() {
    let app = test_app();

    let (status, _) = send(
        &app,
        "PUT",
        "/workouts/999",
        Some(json!({ "name": "Upper" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reorder_workouts_changes_their_order() {
    let app = test_app();
    let microcycle_id = microcycle(&app).await;
    let a = create_workout(&app, microcycle_id, "Push").await;
    let b = create_workout(&app, microcycle_id, "Pull").await;
    let c = create_workout(&app, microcycle_id, "Legs").await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/microcycles/{microcycle_id}/workouts/order"),
        Some(json!({ "ordered_ids": [c, a, b] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/microcycles/{microcycle_id}/workouts"),
        None,
    )
    .await;
    let ids: Vec<i64> = list
        .as_array()
        .expect("list should be an array")
        .iter()
        .map(|w| w["id"].as_i64().expect("id should be a number"))
        .collect();
    assert_eq!(ids, vec![c, a, b]);
}

#[tokio::test]
async fn reorder_with_mismatching_ids_returns_422() {
    let app = test_app();
    let microcycle_id = microcycle(&app).await;
    let a = create_workout(&app, microcycle_id, "Push").await;
    let _b = create_workout(&app, microcycle_id, "Pull").await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/microcycles/{microcycle_id}/workouts/order"),
        Some(json!({ "ordered_ids": [a] })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_workout_removes_it() {
    let app = test_app();
    let microcycle_id = microcycle(&app).await;
    let a = create_workout(&app, microcycle_id, "Push").await;
    let _b = create_workout(&app, microcycle_id, "Pull").await;

    let (status, _) = send(&app, "DELETE", &format!("/workouts/{a}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/microcycles/{microcycle_id}/workouts"),
        None,
    )
    .await;
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn delete_missing_workout_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "DELETE", "/workouts/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
