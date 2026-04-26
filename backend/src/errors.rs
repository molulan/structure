#[derive(Debug, thiserror::Error)]
pub enum MesocycleError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("mesocycle {id} not found")]
    NotFound { id: i64 },
}

#[derive(Debug, thiserror::Error)]
pub enum MicrocycleError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated mesocycle {id} not found")]
    AssociatedMesocycleNotFound { id: i64 },
    #[error("microcycle {id} not found")]
    NotFound { id: i64 },
}

#[derive(Debug, thiserror::Error)]
pub enum WorkoutError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated microcycle {id} not found")]
    AssociatedMicrocycleNotFound { id: i64 },
    #[error("workout {id} not found")]
    NotFound { id: i64 },
}
