use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, put};
use structure_core::domain::planning::Microcycle;
use structure_core::persistence::microcycles as db;
use structure_core::persistence::store::Store;

use crate::dto::ReorderRequest;
use crate::error::ApiError;

pub fn routes() -> Router<Store> {
    Router::new()
        .route(
            "/mesocycles/{mesocycle_id}/microcycles",
            get(list).post(create),
        )
        .route("/mesocycles/{mesocycle_id}/microcycles/order", put(reorder))
        .route("/microcycles/{id}", delete(delete_one))
}

async fn list(
    State(store): State<Store>,
    Path(mesocycle_id): Path<i64>,
) -> Result<Json<Vec<Microcycle>>, ApiError> {
    let microcycles = store.with_conn(|conn| db::list(conn, mesocycle_id))?;
    Ok(Json(microcycles))
}

async fn create(
    State(store): State<Store>,
    Path(mesocycle_id): Path<i64>,
) -> Result<(StatusCode, Json<Microcycle>), ApiError> {
    let microcycle = store.with_conn(|conn| db::create(conn, mesocycle_id))?;
    Ok((StatusCode::CREATED, Json(microcycle)))
}

async fn reorder(
    State(store): State<Store>,
    Path(mesocycle_id): Path<i64>,
    Json(body): Json<ReorderRequest>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::reorder(conn, mesocycle_id, &body.ordered_ids))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::delete(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
