use crate::{dto::LibraryExerciseRequest, error::ApiError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use structure_core::{
    domain::planning::LibraryExercise,
    persistence::{library_exercises as db, store::Store},
};

pub fn routes() -> Router<Store> {
    Router::new()
        .route("/library-exercises", get(list).post(create))
        .route("/library-exercises/{id}", put(update).delete(delete_one))
}

async fn list(State(store): State<Store>) -> Result<Json<Vec<LibraryExercise>>, ApiError> {
    let exercises = store.with_conn(|conn| db::list(conn))?;
    Ok(Json(exercises))
}

async fn create(
    State(store): State<Store>,
    Json(body): Json<LibraryExerciseRequest>,
) -> Result<(StatusCode, Json<LibraryExercise>), ApiError> {
    let exercise_type = body.exercise_type.into();
    let exercise = store.with_conn(|conn| db::create(conn, &body.name, exercise_type))?;
    Ok((StatusCode::CREATED, Json(exercise)))
}

async fn update(
    State(store): State<Store>,
    Path(id): Path<i64>,
    Json(body): Json<LibraryExerciseRequest>,
) -> Result<Json<LibraryExercise>, ApiError> {
    let exercise_type = body.exercise_type.into();
    let exercise = store.with_conn(|conn| db::update(conn, id, &body.name, exercise_type))?;
    Ok(Json(exercise))
}

async fn delete_one(
    State(store): State<Store>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    store.with_conn(|conn| db::delete(conn, id))?;
    Ok(StatusCode::NO_CONTENT)
}
