use crate::{
    dto::{PlannedExerciseRequest, ReorderRequest},
    error::ApiError,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, put},
};
use structure_core::{
    domain::planning::PlannedExercise,
    persistence::{planned_exercises as db, store::Store},
};

pub fn routes() -> Router<Store> {
    Router::new()
        .route(
            "/workouts/{workout_id}/planned-exercises",
            get(list).post(create),
        )
        .route(
            "/workouts/{workout_id}/planned-exercises/order",
            put(reorder),
        )
        .route("/planned-exercises/{id}", delete(delete_one))
}

async fn list(
    State(store): State<Store>,
    Path(workout_id): Path<i64>,
) -> Result<Json<Vec<PlannedExercise>>, ApiError> {
    let planned = store.with_conn(|conn| db::list(conn, workout_id))?;
    Ok(Json(planned))
}

async fn create(
    State(store): State<Store>,
    Path(workout_id): Path<i64>,
    Json(body): Json<PlannedExerciseRequest>,
) -> Result<(StatusCode, Json<PlannedExercise>), ApiError> {
    let planned = store.with_conn(|conn| db::create(conn, workout_id, body.library_exercise_id))?;
    Ok((StatusCode::CREATED, Json(planned)))
}

async fn reorder(
    State(store): State<Store>,
    Path(workout_id): Path<i64>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::reorder(conn, workout_id, &body.ordered_ids))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::delete(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
