#[derive(Debug, thiserror::Error)]
pub enum MicrocycleError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("mesocycle {id} not found")]
    MesocycleNotFound { id: i64 },
}