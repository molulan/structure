mod common;

use axum::Router;
use axum::http::StatusCode;
use common::{
    create_library_exercise, create_microcycle, create_planned_exercise, create_program,
    create_workout, send, test_app,
};
use serde_json::json;

async fn workout(app: &Router) -> i64 {
    let mesocycle_id = create_program(app).await;
    let microcycle_id = create_microcycle(app, mesocycle_id).await;
    create_workout(app, microcycle_id, "Push").await
}

#[tokio::test]
async fn created_planned_exercise_appears_in_list() {
    let app = test_app();
    let workout_id = workout(&app).await;
    let exercise_id = create_library_exercise(&app, "Bench Press", "Weighted").await;

    let (status, created) = send(
        &app,
        "POST",
        &format!("/workouts/{workout_id}/planned-exercises"),
        Some(json!({ "library_exercise_id": exercise_id })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["position"], 0);
    assert_eq!(created["exercise"]["name"], "Bench Press");

    let (status, list) = send(
        &app,
        "GET",
        &format!("/workouts/{workout_id}/planned-exercises"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn create_in_missing_workout_returns_404() {
    let app = test_app();
    let exercise_id = create_library_exercise(&app, "Squat", "Weighted").await;

    let (status, _) = send(
        &app,
        "POST",
        "/workouts/999/planned-exercises",
        Some(json!({ "library_exercise_id": exercise_id })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_with_missing_library_exercise_returns_422() {
    let app = test_app();
    let workout_id = workout(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!("/workouts/{workout_id}/planned-exercises"),
        Some(json!({ "library_exercise_id": 999 })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn reorder_planned_exercises_changes_their_order() {
    let app = test_app();
    let workout_id = workout(&app).await;
    let exercise_id = create_library_exercise(&app, "Bench Press", "Weighted").await;
    let a = create_planned_exercise(&app, workout_id, exercise_id).await;
    let b = create_planned_exercise(&app, workout_id, exercise_id).await;
    let c = create_planned_exercise(&app, workout_id, exercise_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/workouts/{workout_id}/planned-exercises/order"),
        Some(json!({ "ordered_ids": [c, a, b] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/workouts/{workout_id}/planned-exercises"),
        None,
    )
    .await;
    let ids: Vec<i64> = list
        .as_array()
        .expect("list should be an array")
        .iter()
        .map(|p| p["id"].as_i64().expect("id should be a number"))
        .collect();
    assert_eq!(ids, vec![c, a, b]);
}

#[tokio::test]
async fn reorder_with_mismatching_ids_returns_422() {
    let app = test_app();
    let workout_id = workout(&app).await;
    let exercise_id = create_library_exercise(&app, "Bench Press", "Weighted").await;
    let a = create_planned_exercise(&app, workout_id, exercise_id).await;
    let _b = create_planned_exercise(&app, workout_id, exercise_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/workouts/{workout_id}/planned-exercises/order"),
        Some(json!({ "ordered_ids": [a] })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_planned_exercise_removes_it() {
    let app = test_app();
    let workout_id = workout(&app).await;
    let exercise_id = create_library_exercise(&app, "Bench Press", "Weighted").await;
    let a = create_planned_exercise(&app, workout_id, exercise_id).await;
    let _b = create_planned_exercise(&app, workout_id, exercise_id).await;

    let (status, _) = send(&app, "DELETE", &format!("/planned-exercises/{a}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/workouts/{workout_id}/planned-exercises"),
        None,
    )
    .await;
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn delete_missing_planned_exercise_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "DELETE", "/planned-exercises/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
