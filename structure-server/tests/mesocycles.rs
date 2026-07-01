mod common;

use axum::http::StatusCode;
use common::{
    create_library_exercise, create_microcycle, create_planned_exercise, create_program,
    create_set, create_workout, send, test_app,
};
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

#[tokio::test]
async fn get_full_mesocycle_returns_the_whole_tree() {
    let app = test_app();
    let mesocycle_id = create_program(&app).await;
    let microcycle_id = create_microcycle(&app, mesocycle_id).await;
    let workout_id = create_workout(&app, microcycle_id, "Push").await;
    let exercise_id = create_library_exercise(&app, "Bench Press", "Weighted").await;
    let planned_id = create_planned_exercise(&app, workout_id, exercise_id).await;
    let set_id = create_set(&app, planned_id).await;

    let (status, full) = send(
        &app,
        "GET",
        &format!("/mesocycles/{mesocycle_id}/full"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(full["id"].as_i64(), Some(mesocycle_id));
    let microcycle = &full["microcycles"][0];
    assert_eq!(microcycle["id"].as_i64(), Some(microcycle_id));
    let workout = &microcycle["workouts"][0];
    assert_eq!(workout["id"].as_i64(), Some(workout_id));
    assert_eq!(workout["name"], "Push");
    let planned = &workout["planned_exercises"][0];
    assert_eq!(planned["id"].as_i64(), Some(planned_id));
    assert_eq!(planned["exercise"]["name"], "Bench Press");
    assert_eq!(planned["sets"][0]["id"].as_i64(), Some(set_id));
    // The new set-group prescription layer is exposed alongside the legacy
    // sets; no create-set-group route exists yet, so it serializes as empty.
    assert_eq!(planned["set_groups"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn get_full_missing_mesocycle_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "GET", "/mesocycles/999/full", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
