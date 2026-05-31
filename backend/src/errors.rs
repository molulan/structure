use crate::domain::planning::PlannedExerciseValidationError;

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

#[derive(Debug, thiserror::Error)]
pub enum PlannedExerciseError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated workout {id} not found")]
    AssociatedWorkoutNotFound { id: i64 },
    #[error("associated exercise {id} not found")]
    AssociatedExerciseNotFound { id: i64 },
    #[error("planned exercise {id} not found")]
    NotFound { id: i64 },
    #[error("data corruption: {0}")]
    DataCorruption(String),
    #[error("set error: {0}")]
    SetOperation(#[from] SetError),
    #[error("validation error: {0}")]
    ValidationError(#[from] PlannedExerciseValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ExerciseError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("exercise with name '{name}' already exists")]
    DuplicateName { name: String },
    #[error("exercise {id} not found")]
    NotFound { id: i64 },
}

#[derive(Debug, thiserror::Error)]
pub enum SetError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated planned exercise {id} not found")]
    AssociatedPlannedExerciseNotFound { id: i64 },
    #[error("planned set {id} not found")]
    NotFound { id: i64 },
}
