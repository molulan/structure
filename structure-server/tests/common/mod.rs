// Each integration-test binary uses a different subset of these helpers.
#![allow(dead_code)]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use structure_core::persistence::store::Store;
use tower::ServiceExt;

/// Builds the full application over a fresh in-memory database.
pub fn test_app() -> Router {
    let store = Store::open(":memory:").expect("in-memory store should open");
    structure_server::router(store)
}

/// Sends one request through the router and returns its status and JSON body
/// (`Null` when the body is empty, e.g. a 204).
pub async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(value) => builder
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&value).expect("body should serialize"),
            )),
        None => builder.body(Body::empty()),
    }
    .expect("request should build");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("request should succeed");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("body should be json")
    };
    (status, value)
}

pub async fn create_program(app: &Router) -> i64 {
    let (_, created) = send(
        app,
        "POST",
        "/mesocycles",
        Some(json!({ "name": "Program", "mode": "Manual" })),
    )
    .await;
    created["id"].as_i64().expect("id should be a number")
}

pub async fn create_microcycle(app: &Router, mesocycle_id: i64) -> i64 {
    let (_, created) = send(
        app,
        "POST",
        &format!("/mesocycles/{mesocycle_id}/microcycles"),
        None,
    )
    .await;
    created["id"].as_i64().expect("id should be a number")
}

pub async fn create_workout(app: &Router, microcycle_id: i64, name: &str) -> i64 {
    let (_, created) = send(
        app,
        "POST",
        &format!("/microcycles/{microcycle_id}/workouts"),
        Some(json!({ "name": name })),
    )
    .await;
    created["id"].as_i64().expect("id should be a number")
}

pub async fn create_library_exercise(app: &Router, name: &str, exercise_type: &str) -> i64 {
    let (_, created) = send(
        app,
        "POST",
        "/library-exercises",
        Some(json!({ "name": name, "exercise_type": exercise_type })),
    )
    .await;
    created["id"].as_i64().expect("id should be a number")
}

pub async fn create_planned_exercise(
    app: &Router,
    workout_id: i64,
    library_exercise_id: i64,
) -> i64 {
    let (_, created) = send(
        app,
        "POST",
        &format!("/workouts/{workout_id}/planned-exercises"),
        Some(json!({ "library_exercise_id": library_exercise_id })),
    )
    .await;
    created["id"].as_i64().expect("id should be a number")
}

/// Creates a plain bodyweight set (10 reps, regular, no effort) and returns its id.
pub async fn create_set(app: &Router, planned_exercise_id: i64) -> i64 {
    let (_, created) = send(
        app,
        "POST",
        &format!("/planned-exercises/{planned_exercise_id}/sets"),
        Some(json!({ "load": "Bodyweight", "reps": 10, "set_type": { "Regular": { "effort": null } } })),
    )
    .await;
    created["id"].as_i64().expect("id should be a number")
}
