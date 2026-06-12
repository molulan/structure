mod common;

use axum::Router;
use axum::http::StatusCode;
use common::{
    create_library_exercise, create_microcycle, create_planned_exercise, create_program,
    create_set, create_workout, send, test_app,
};
use serde_json::json;

async fn planned_exercise(app: &Router) -> i64 {
    let mesocycle_id = create_program(app).await;
    let microcycle_id = create_microcycle(app, mesocycle_id).await;
    let workout_id = create_workout(app, microcycle_id, "Push").await;
    let exercise_id = create_library_exercise(app, "Bench Press", "Weighted").await;
    create_planned_exercise(app, workout_id, exercise_id).await
}

#[tokio::test]
async fn created_set_appears_in_list() {
    let app = test_app();
    let planned_exercise_id = planned_exercise(&app).await;

    let (status, created) = send(
        &app,
        "POST",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        Some(json!({
            "load": { "Weighted": { "weight": { "value": 100.0, "unit": "Kg" } } },
            "reps": 5,
            "set_type": { "Regular": { "effort": { "Rir": 2 } } }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["position"], 0);
    assert_eq!(created["reps"], 5);
    assert_eq!(
        created["load"]["Weighted"]["weight"]["value"].as_f64(),
        Some(100.0)
    );
    assert_eq!(created["set_type"]["Regular"]["effort"]["Rir"], 2);

    let (status, list) = send(
        &app,
        "GET",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn create_set_with_out_of_range_rpe_returns_422() {
    let app = test_app();
    let planned_exercise_id = planned_exercise(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        Some(json!({
            "load": "Bodyweight",
            "reps": 8,
            "set_type": { "Regular": { "effort": { "Rpe": 99 } } }
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_set_with_load_not_matching_exercise_returns_422() {
    let app = test_app();
    // The planned exercise uses the Weighted "Bench Press".
    let planned_exercise_id = planned_exercise(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        Some(json!({
            "load": "Bodyweight",
            "reps": 10,
            "set_type": { "Regular": { "effort": null } }
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_set_in_missing_planned_exercise_returns_404() {
    let app = test_app();

    let (status, _) = send(
        &app,
        "POST",
        "/planned-exercises/999/sets",
        Some(json!({ "load": "Bodyweight", "reps": 10, "set_type": { "Regular": { "effort": null } } })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_changes_the_set() {
    let app = test_app();
    let planned_exercise_id = planned_exercise(&app).await;
    let id = create_set(&app, planned_exercise_id).await;

    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/sets/{id}"),
        Some(json!({
            "load": { "Weighted": { "weight": { "value": 60.0, "unit": "Kg" } } },
            "reps": 12,
            "set_type": "Myorep"
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["reps"], 12);
    assert_eq!(updated["set_type"], "Myorep");
    assert_eq!(
        updated["load"]["Weighted"]["weight"]["value"].as_f64(),
        Some(60.0)
    );
}

#[tokio::test]
async fn reorder_sets_changes_their_order() {
    let app = test_app();
    let planned_exercise_id = planned_exercise(&app).await;
    let a = create_set(&app, planned_exercise_id).await;
    let b = create_set(&app, planned_exercise_id).await;
    let c = create_set(&app, planned_exercise_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/planned-exercises/{planned_exercise_id}/sets/order"),
        Some(json!({ "ordered_ids": [c, a, b] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        None,
    )
    .await;
    let ids: Vec<i64> = list
        .as_array()
        .expect("list should be an array")
        .iter()
        .map(|s| s["id"].as_i64().expect("id should be a number"))
        .collect();
    assert_eq!(ids, vec![c, a, b]);
}

#[tokio::test]
async fn reorder_with_mismatching_ids_returns_422() {
    let app = test_app();
    let planned_exercise_id = planned_exercise(&app).await;
    let a = create_set(&app, planned_exercise_id).await;
    let _b = create_set(&app, planned_exercise_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/planned-exercises/{planned_exercise_id}/sets/order"),
        Some(json!({ "ordered_ids": [a] })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_set_removes_it() {
    let app = test_app();
    let planned_exercise_id = planned_exercise(&app).await;
    let a = create_set(&app, planned_exercise_id).await;
    let _b = create_set(&app, planned_exercise_id).await;

    let (status, _) = send(&app, "DELETE", &format!("/sets/{a}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        None,
    )
    .await;
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn delete_missing_set_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "DELETE", "/sets/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
