mod dto;
mod error;
mod library_exercises;
mod mesocycles;
mod microcycles;
mod planned_exercises;
mod set_groups;
mod workouts;

pub use error::ApiErrorBody;

use std::path::PathBuf;

use axum::{
    Router,
    routing::{any, get},
};
use structure_core::persistence::store::Store;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::error::ApiError;

fn api_routes() -> Router<Store> {
    Router::new()
        .merge(mesocycles::routes())
        .merge(microcycles::routes())
        .merge(workouts::routes())
        .merge(library_exercises::routes())
        .merge(planned_exercises::routes())
        .merge(set_groups::routes())
        .fallback(api_not_found)
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("no such endpoint")
}

/// The HTTP API: `/health` at the root, the entity routes under `/api`.
///
/// The `/api` prefix keeps the API disjoint from the SPA's client routes, which
/// share paths with it: `/mesocycles/5` is a page [`app`] serves the shell for,
/// not the `GET /mesocycles/{id}` JSON endpoint at `/api/mesocycles/5`.
pub fn router(store: Store) -> Router {
    Router::new()
        .route("/health", get(health))
        // Axum's nest doesn't route the bare `/api/` inside, so guard it here
        // (any method) to keep it a JSON 404 rather than the SPA fallback.
        .route("/api/", any(api_not_found))
        .nest("/api", api_routes())
        .layer(CorsLayer::permissive())
        .with_state(store)
}

/// [`router`] plus the built web app served for every non-API path, when
/// `web_dir` is given.
pub fn app(store: Store, web_dir: Option<PathBuf>) -> Router {
    match web_dir {
        Some(dir) => {
            // `fallback`, not `not_found_service`: a deep link must get index.html
            // with a 200 to boot the app, not a 404.
            let serve = ServeDir::new(&dir).fallback(ServeFile::new(dir.join("index.html")));
            router(store).fallback_service(serve)
        }
        None => router(store),
    }
}

async fn health() -> &'static str {
    "ok"
}
