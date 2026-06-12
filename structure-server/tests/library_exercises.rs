mod common;

use axum::http::StatusCode;
use common::{
    create_library_exercise, create_microcycle, create_planned_exercise, create_program,
    create_workout, send, test_app,
};
use serde_json::json;

#[tokio::test]
async fn created_library_exercise_appears_in_list() {
    let app = test_app();

    let (status, created) = send(
        &app,
        "POST",
        "/library-exercises",
        Some(json!({ "name": "Bench Press", "exercise_type": "Weighted" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["name"], "Bench Press");
    assert_eq!(created["exercise_type"], "Weighted");

    let (status, list) = send(&app, "GET", "/library-exercises", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn create_with_duplicate_name_returns_409() {
    let app = test_app();
    send(
        &app,
        "POST",
        "/library-exercises",
        Some(json!({ "name": "Squat", "exercise_type": "Weighted" })),
    )
    .await;

    let (status, body) = send(
        &app,
        "POST",
        "/library-exercises",
        Some(json!({ "name": "Squat", "exercise_type": "Bodyweight" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn create_with_empty_name_returns_422() {
    let app = test_app();

    let (status, _) = send(
        &app,
        "POST",
        "/library-exercises",
        Some(json!({ "name": "", "exercise_type": "Weighted" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn update_changes_name_and_type() {
    let app = test_app();
    let id = create_library_exercise(&app, "Bench Press", "Weighted").await;

    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/library-exercises/{id}"),
        Some(json!({ "name": "Incline Press", "exercise_type": "Bodyweight" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["name"], "Incline Press");
    assert_eq!(updated["exercise_type"], "Bodyweight");
}

#[tokio::test]
async fn update_to_a_name_taken_by_another_returns_409() {
    let app = test_app();
    create_library_exercise(&app, "Squat", "Weighted").await;
    let id = create_library_exercise(&app, "Bench Press", "Weighted").await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/library-exercises/{id}"),
        Some(json!({ "name": "Squat", "exercise_type": "Weighted" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn update_missing_library_exercise_returns_404() {
    let app = test_app();

    let (status, _) = send(
        &app,
        "PUT",
        "/library-exercises/999",
        Some(json!({ "name": "X", "exercise_type": "Weighted" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_library_exercise_removes_it() {
    let app = test_app();
    let id = create_library_exercise(&app, "Squat", "Weighted").await;

    let (status, _) = send(&app, "DELETE", &format!("/library-exercises/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(&app, "GET", "/library-exercises", None).await;
    assert_eq!(list.as_array().expect("list should be an array").len(), 0);
}

#[tokio::test]
async fn delete_missing_library_exercise_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "DELETE", "/library-exercises/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_library_exercise_in_use_returns_409() {
    let app = test_app();
    let exercise_id = create_library_exercise(&app, "Bench Press", "Weighted").await;
    let mesocycle_id = create_program(&app).await;
    let microcycle_id = create_microcycle(&app, mesocycle_id).await;
    let workout_id = create_workout(&app, microcycle_id, "Push").await;
    create_planned_exercise(&app, workout_id, exercise_id).await;

    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/library-exercises/{exercise_id}"),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}
