use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use structure_core::domain::planning::Workout;
use structure_core::persistence::store::Store;
use structure_core::persistence::workouts as db;

use crate::dto::{ReorderRequest, WorkoutNameRequest};
use crate::error::ApiError;

pub fn routes() -> Router<Store> {
    Router::new()
        .route(
            "/microcycles/{microcycle_id}/workouts",
            get(list).post(create),
        )
        .route("/microcycles/{microcycle_id}/workouts/order", put(reorder))
        .route("/workouts/{id}", put(rename).delete(delete_one))
}

async fn list(
    State(store): State<Store>,
    Path(microcycle_id): Path<i64>,
) -> Result<Json<Vec<Workout>>, ApiError> {
    let workouts = store.with_conn(|conn| db::list_workouts(conn, microcycle_id))?;
    Ok(Json(workouts))
}

async fn create(
    State(store): State<Store>,
    Path(microcycle_id): Path<i64>,
    Json(body): Json<WorkoutNameRequest>,
) -> Result<(StatusCode, Json<Workout>), ApiError> {
    let workout = store.with_conn(|conn| db::create_workout(conn, microcycle_id, &body.name))?;
    Ok((StatusCode::CREATED, Json(workout)))
}

async fn rename(
    State(store): State<Store>,
    Path(id): Path<i64>,
    Json(body): Json<WorkoutNameRequest>,
) -> Result<Json<Workout>, ApiError> {
    let workout = store.with_conn(|conn| db::update_workout(conn, id, &body.name))?;
    Ok(Json(workout))
}

async fn reorder(
    State(store): State<Store>,
    Path(microcycle_id): Path<i64>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::reorder_workouts(conn, microcycle_id, &body.ordered_ids))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::delete_workout(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
