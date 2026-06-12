use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use structure_core::persistence::exercises::{LibraryExerciseError, PlannedExerciseError};
use structure_core::persistence::mesocycles::MesocycleError;
use structure_core::persistence::microcycles::MicrocycleError;
use structure_core::persistence::sets::SetError;
use structure_core::persistence::workouts::WorkoutError;

/// An error rendered as a JSON `{ "error": ... }` body with a status code.
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        ApiError {
            status,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::NOT_FOUND, message)
    }

    pub fn unprocessable(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        ApiError::new(StatusCode::CONFLICT, message)
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        eprintln!("internal error: {error}");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<MesocycleError> for ApiError {
    fn from(error: MesocycleError) -> Self {
        match error {
            MesocycleError::NotFound { id } => {
                ApiError::not_found(format!("mesocycle {id} not found"))
            }
            MesocycleError::Database(error) => ApiError::internal(error),
            MesocycleError::InvalidName(error) => ApiError::unprocessable(error.to_string()),
        }
    }
}

impl From<MicrocycleError> for ApiError {
    fn from(error: MicrocycleError) -> Self {
        match error {
            MicrocycleError::Database(error) => ApiError::internal(error),
            MicrocycleError::AssociatedMesocycleNotFound { id } => {
                ApiError::not_found(format!("mesocycle {id} not found"))
            }
            MicrocycleError::NotFound { id } => {
                ApiError::not_found(format!("microcycle {id} not found"))
            }
            MicrocycleError::ReorderMismatch { mesocycle_id } => ApiError::unprocessable(format!(
                "reorder list does not match the microcycles of mesocycle {mesocycle_id}"
            )),
        }
    }
}

impl From<WorkoutError> for ApiError {
    fn from(error: WorkoutError) -> Self {
        match error {
            WorkoutError::Database(error) => ApiError::internal(error),
            WorkoutError::AssociatedMicrocycleNotFound { id } => {
                ApiError::not_found(format!("microcycle {id} not found"))
            }
            WorkoutError::NotFound { id } => ApiError::not_found(format!("workout {id} not found")),
            WorkoutError::ReorderMismatch { microcycle_id } => ApiError::unprocessable(format!(
                "reorder list does not match the workouts of microcycle {microcycle_id}"
            )),
            WorkoutError::InvalidName(error) => ApiError::unprocessable(error.to_string()),
        }
    }
}

impl From<LibraryExerciseError> for ApiError {
    fn from(error: LibraryExerciseError) -> Self {
        match error {
            LibraryExerciseError::Database(error) => ApiError::internal(error),
            LibraryExerciseError::DuplicateName { name } => {
                ApiError::conflict(format!("an exercise named '{name}' already exists"))
            }
            LibraryExerciseError::NotFound { id } => {
                ApiError::not_found(format!("library exercise {id} not found"))
            }
            LibraryExerciseError::InUse { id } => ApiError::conflict(format!(
                "library exercise {id} is used by one or more planned exercises"
            )),
            LibraryExerciseError::InvalidName(error) => ApiError::unprocessable(error.to_string()),
        }
    }
}

impl From<PlannedExerciseError> for ApiError {
    fn from(error: PlannedExerciseError) -> Self {
        match error {
            PlannedExerciseError::Database(error) => ApiError::internal(error),
            PlannedExerciseError::AssociatedWorkoutNotFound { id } => {
                ApiError::not_found(format!("workout {id} not found"))
            }
            PlannedExerciseError::AssociatedExerciseNotFound { id } => {
                ApiError::unprocessable(format!("library exercise {id} does not exist"))
            }
            PlannedExerciseError::NotFound { id } => {
                ApiError::not_found(format!("planned exercise {id} not found"))
            }
            PlannedExerciseError::ReorderMismatch { workout_id } => {
                ApiError::unprocessable(format!(
                    "reorder list does not match the planned exercises of workout {workout_id}"
                ))
            }
        }
    }
}

impl From<SetError> for ApiError {
    fn from(error: SetError) -> Self {
        match error {
            SetError::Database(error) => ApiError::internal(error),
            SetError::AssociatedPlannedExerciseNotFound { id } => {
                ApiError::not_found(format!("planned exercise {id} not found"))
            }
            SetError::NotFound { id } => ApiError::not_found(format!("set {id} not found")),
            SetError::ReorderMismatch {
                planned_exercise_id,
            } => ApiError::unprocessable(format!(
                "reorder list does not match the sets of planned exercise {planned_exercise_id}"
            )),
            SetError::Invalid(error) => ApiError::unprocessable(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_404_with_a_message() {
        let api_error = ApiError::from(MesocycleError::NotFound { id: 7 });

        assert_eq!(api_error.status, StatusCode::NOT_FOUND);
        assert!(api_error.message.contains('7'));
    }
}
