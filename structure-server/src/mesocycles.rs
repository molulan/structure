use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use structure_core::domain::planning::Mesocycle;
use structure_core::persistence::aggregates::{FullMesocycle, get_full_mesocycle};
use structure_core::persistence::mesocycles::{self as db, MesocycleRow};
use structure_core::persistence::store::Store;

use crate::dto::{CreateMesocycleRequest, UpdateMesocycleRequest};
use crate::error::ApiError;

pub fn routes() -> Router<Store> {
    Router::new()
        .route("/mesocycles", get(list).post(create))
        .route(
            "/mesocycles/{id}",
            get(get_one).put(update).delete(delete_one),
        )
        .route("/mesocycles/{id}/full", get(get_full))
}

async fn list(State(store): State<Store>) -> Result<Json<Vec<MesocycleRow>>, ApiError> {
    let mesocycles = store.with_conn(|conn| db::list_mesocycles(conn))?;
    Ok(Json(mesocycles))
}

async fn create(
    State(store): State<Store>,
    Json(body): Json<CreateMesocycleRequest>,
) -> Result<(StatusCode, Json<Mesocycle>), ApiError> {
    let mode = body.mode.into();
    let mesocycle = store.with_conn(|conn| db::create_mesocycle(conn, &body.name, mode))?;
    Ok((StatusCode::CREATED, Json(mesocycle)))
}

async fn get_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<Json<MesocycleRow>, ApiError> {
    let mesocycle = store
        .with_conn(|conn| db::get_mesocycle(conn, id))?
        .ok_or_else(|| ApiError::not_found(format!("mesocycle {id} not found")))?;
    Ok(Json(mesocycle))
}

async fn get_full(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<Json<FullMesocycle>, ApiError> {
    let mesocycle = store
        .with_conn(|conn| get_full_mesocycle(conn, id))?
        .ok_or_else(|| ApiError::not_found(format!("mesocycle {id} not found")))?;
    Ok(Json(mesocycle))
}

async fn update(
    State(store): State<Store>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateMesocycleRequest>,
) -> Result<Json<Mesocycle>, ApiError> {
    let mode = body.mode.into();
    let mesocycle = store.with_conn(|conn| db::update_mesocycle(conn, id, &body.name, mode))?;
    Ok(Json(mesocycle))
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::delete_mesocycle(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
