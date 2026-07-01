mod dto;
mod error;
mod library_exercises;
mod mesocycles;
mod microcycles;
mod planned_exercises;
mod set_groups;
mod sets;
mod workouts;

use axum::{Router, routing::get};
use structure_core::persistence::store::Store;
use tower_http::cors::CorsLayer;

/// Builds the HTTP application over a [`Store`]. Kept separate from `main` so
/// integration tests can construct the router against an in-memory database.
pub fn router(store: Store) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(mesocycles::routes())
        .merge(microcycles::routes())
        .merge(workouts::routes())
        .merge(library_exercises::routes())
        .merge(planned_exercises::routes())
        .merge(sets::routes())
        .merge(set_groups::routes())
        .layer(CorsLayer::permissive())
        .with_state(store)
}

async fn health() -> &'static str {
    "ok"
}
