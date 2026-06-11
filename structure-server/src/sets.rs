use crate::{
    dto::{ReorderRequest, SetRequest},
    error::ApiError,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use structure_core::{
    domain::planning::{Set, SetType},
    persistence::{sets, store::Store},
};

pub fn routes() -> Router<Store> {
    Router::new()
        .route(
            "/planned-exercises/{planned_exercise_id}/sets",
            get(list).post(create),
        )
        .route(
            "/planned-exercises/{planned_exercise_id}/sets/order",
            put(reorder),
        )
        .route("/sets/{id}", put(update).delete(delete_one))
}

async fn list(
    State(store): State<Store>,
    Path(planned_exercise_id): Path<i64>,
) -> Result<Json<Vec<Set>>, ApiError> {
    let sets = store.with_conn(|conn| sets::list_planned_sets(conn, planned_exercise_id))?;
    Ok(Json(sets))
}

async fn create(
    State(store): State<Store>,
    Path(planned_exercise_id): Path<i64>,
    Json(body): Json<SetRequest>,
) -> Result<(StatusCode, Json<Set>), ApiError> {
    let load = body.load.into();
    let set_type = SetType::try_from(body.set_type)
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    let reps = body.reps;
    let set = store.with_conn(|conn| {
        sets::create_planned_set(conn, planned_exercise_id, load, reps, set_type)
    })?;
    Ok((StatusCode::CREATED, Json(set)))
}

async fn update(
    State(store): State<Store>,
    Path(id): Path<i64>,
    Json(body): Json<SetRequest>,
) -> Result<Json<Set>, ApiError> {
    let load = body.load.into();
    let set_type = SetType::try_from(body.set_type)
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    let reps = body.reps;
    let set = store.with_conn(|conn| sets::update_planned_set(conn, id, load, reps, set_type))?;
    Ok(Json(set))
}

async fn reorder(
    State(store): State<Store>,
    Path(planned_exercise_id): Path<i64>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| {
        sets::reorder_planned_sets(conn, planned_exercise_id, &body.ordered_ids)
    })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| sets::delete_planned_set(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
