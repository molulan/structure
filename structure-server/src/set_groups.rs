use crate::{
    dto::{ReorderRequest, SetGroupRequest},
    error::ApiError,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use structure_core::{
    domain::planning::{SetGroup, SetGroupType},
    persistence::{set_groups, store::Store},
};

pub fn routes() -> Router<Store> {
    Router::new()
        .route(
            "/planned-exercises/{planned_exercise_id}/set-groups",
            get(list).post(create),
        )
        .route(
            "/planned-exercises/{planned_exercise_id}/set-groups/order",
            put(reorder),
        )
        .route("/set-groups/{id}", put(update).delete(delete_one))
}

async fn list(
    State(store): State<Store>,
    Path(planned_exercise_id): Path<i64>,
) -> Result<Json<Vec<SetGroup>>, ApiError> {
    let set_groups = store.with_conn(|conn| set_groups::list(conn, planned_exercise_id))?;
    Ok(Json(set_groups))
}

async fn create(
    State(store): State<Store>,
    Path(planned_exercise_id): Path<i64>,
    Json(body): Json<SetGroupRequest>,
) -> Result<(StatusCode, Json<SetGroup>), ApiError> {
    let set_group_type = SetGroupType::try_from(body.set_group_type)
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    let set_group = store.with_conn(|conn| {
        set_groups::create(
            conn,
            planned_exercise_id,
            body.number_of_sets,
            set_group_type,
        )
    })?;
    Ok((StatusCode::CREATED, Json(set_group)))
}

async fn update(
    State(store): State<Store>,
    Path(id): Path<i64>,
    Json(body): Json<SetGroupRequest>,
) -> Result<Json<SetGroup>, ApiError> {
    let set_group_type = SetGroupType::try_from(body.set_group_type)
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    let set_group = store
        .with_conn(|conn| set_groups::update(conn, id, body.number_of_sets, set_group_type))?;
    Ok(Json(set_group))
}

async fn reorder(
    State(store): State<Store>,
    Path(planned_exercise_id): Path<i64>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| set_groups::reorder(conn, planned_exercise_id, &body.ordered_ids))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| set_groups::delete(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
