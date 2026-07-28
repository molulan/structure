mod common;

use axum::Router;
use axum::http::StatusCode;
use common::{
    create_library_exercise, create_microcycle, create_planned_exercise, create_program,
    create_set_group, create_workout, send, test_app,
};
use serde_json::json;

/// Returns the planned exercise id and a microcycle id: a set group lives in the
/// (planned exercise, microcycle) cell.
async fn planned_exercise(app: &Router) -> (i64, i64) {
    let mesocycle_id = create_program(app).await;
    let microcycle_id = create_microcycle(app, mesocycle_id).await;
    let workout_id = create_workout(app, mesocycle_id, "Push").await;
    let exercise_id = create_library_exercise(app, "Bench Press", "Weighted").await;
    let planned_id = create_planned_exercise(app, workout_id, exercise_id).await;
    (planned_id, microcycle_id)
}

#[tokio::test]
async fn created_set_group_appears_in_list() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    let (status, created) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({
            "number_of_sets": 3,
            "set_group_type": {
                "Prescribed": {
                    "set_type": "Regular",
                    "reps": { "Range": { "min": 8, "max": 12 } },
                    "intensity": { "Rir": 2 }
                }
            }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["position"], 0);
    assert_eq!(created["number_of_sets"], 3);
    let prescribed = &created["set_group_type"]["Prescribed"];
    assert_eq!(prescribed["set_type"], "Regular");
    assert_eq!(prescribed["reps"]["Range"]["min"], 8);
    assert_eq!(prescribed["reps"]["Range"]["max"], 12);
    assert_eq!(prescribed["intensity"]["Rir"], 2);

    let (status, list) = send(
        &app,
        "GET",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn created_set_group_round_trips_an_amrap_rep_target() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    let (status, created) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({
            "number_of_sets": 1,
            "set_group_type": {
                "Prescribed": {
                    "set_type": "Regular",
                    "reps": { "AtLeast": 12 },
                    "intensity": { "Rir": 0 }
                }
            }
        })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        created["set_group_type"]["Prescribed"]["reps"]["AtLeast"],
        12
    );

    let (_, list) = send(
        &app,
        "GET",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        None,
    )
    .await;
    assert_eq!(
        list[0]["set_group_type"]["Prescribed"]["reps"]["AtLeast"],
        12
    );
}

#[tokio::test]
async fn created_myorep_match_set_group_serializes_as_the_bare_variant() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    let (status, created) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({ "number_of_sets": 2, "set_group_type": "MyorepMatch" })),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["set_group_type"], "MyorepMatch");
    assert_eq!(created["number_of_sets"], 2);
}

#[tokio::test]
async fn create_set_group_with_zero_sets_returns_422() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({ "number_of_sets": 0, "set_group_type": "MyorepMatch" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_set_group_with_out_of_range_rir_returns_422() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({
            "number_of_sets": 3,
            "set_group_type": {
                "Prescribed": {
                    "set_type": "Regular",
                    "reps": { "Exact": 5 },
                    "intensity": { "Rir": 99 }
                }
            }
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_set_group_with_non_ascending_reps_returns_422() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({
            "number_of_sets": 3,
            "set_group_type": {
                "Prescribed": {
                    "set_type": "Regular",
                    "reps": { "Range": { "min": 12, "max": 8 } },
                    "intensity": { "Rir": 2 }
                }
            }
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_prescribed_myorep_with_proximity_intensity_returns_422() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;

    // A failure-based set type (Myorep) rejects a proximity-to-failure intensity.
    let (status, _) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        Some(json!({
            "number_of_sets": 3,
            "set_group_type": {
                "Prescribed": {
                    "set_type": "Myorep",
                    "reps": { "Exact": 10 },
                    "intensity": { "Rir": 0 }
                }
            }
        })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_set_group_in_missing_planned_exercise_returns_404() {
    let app = test_app();

    let (status, _) = send(
        &app,
        "POST",
        "/api/planned-exercises/999/microcycles/1/set-groups",
        Some(json!({ "number_of_sets": 3, "set_group_type": "MyorepMatch" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_set_group_in_missing_microcycle_returns_404() {
    let app = test_app();
    let (planned_exercise_id, _microcycle_id) = planned_exercise(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!("/api/planned-exercises/{planned_exercise_id}/microcycles/999/set-groups"),
        Some(json!({ "number_of_sets": 3, "set_group_type": "MyorepMatch" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_set_group_in_a_microcycle_from_another_mesocycle_returns_422() {
    let app = test_app();
    let (planned_exercise_id, _microcycle_id) = planned_exercise(&app).await;

    // A microcycle that exists, but under a different mesocycle — not a valid
    // cell for this planned exercise.
    let other_mesocycle = create_program(&app).await;
    let foreign_microcycle = create_microcycle(&app, other_mesocycle).await;

    let (status, _) = send(
        &app,
        "POST",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{foreign_microcycle}/set-groups"
        ),
        Some(json!({ "number_of_sets": 3, "set_group_type": "MyorepMatch" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn update_changes_the_set_group() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;
    let id = create_set_group(&app, planned_exercise_id, microcycle_id).await;

    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/api/set-groups/{id}"),
        Some(json!({ "number_of_sets": 2, "set_group_type": "MyorepMatch" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["number_of_sets"], 2);
    assert_eq!(updated["set_group_type"], "MyorepMatch");
}

#[tokio::test]
async fn reorder_set_groups_changes_their_order() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;
    let a = create_set_group(&app, planned_exercise_id, microcycle_id).await;
    let b = create_set_group(&app, planned_exercise_id, microcycle_id).await;
    let c = create_set_group(&app, planned_exercise_id, microcycle_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups/order"
        ),
        Some(json!({ "ordered_ids": [c, a, b] })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
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
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;
    let a = create_set_group(&app, planned_exercise_id, microcycle_id).await;
    let _b = create_set_group(&app, planned_exercise_id, microcycle_id).await;

    let (status, _) = send(
        &app,
        "PUT",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups/order"
        ),
        Some(json!({ "ordered_ids": [a] })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_set_group_removes_it() {
    let app = test_app();
    let (planned_exercise_id, microcycle_id) = planned_exercise(&app).await;
    let a = create_set_group(&app, planned_exercise_id, microcycle_id).await;
    let _b = create_set_group(&app, planned_exercise_id, microcycle_id).await;

    let (status, _) = send(&app, "DELETE", &format!("/api/set-groups/{a}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, list) = send(
        &app,
        "GET",
        &format!(
            "/api/planned-exercises/{planned_exercise_id}/microcycles/{microcycle_id}/set-groups"
        ),
        None,
    )
    .await;
    assert_eq!(list.as_array().expect("list should be an array").len(), 1);
}

#[tokio::test]
async fn delete_missing_set_group_returns_404() {
    let app = test_app();

    let (status, _) = send(&app, "DELETE", "/api/set-groups/999", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
