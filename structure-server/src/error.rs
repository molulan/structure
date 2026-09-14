use crate::dto::SetGroupTypeInputError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use structure_core::persistence::aggregates::FullMesocycleError;
use structure_core::persistence::library_exercises::LibraryExerciseError;
use structure_core::persistence::mesocycles::MesocycleError;
use structure_core::persistence::microcycles::MicrocycleError;
use structure_core::persistence::planned_exercises::PlannedExerciseError;
use structure_core::persistence::set_groups::SetGroupError;
use structure_core::persistence::workouts::WorkoutError;
use ts_rs::TS;

/// The JSON body of an [`ApiError`] response.
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApiErrorBody {
    pub error: String,
}

/// The status and message a handler returns when a request fails.
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
        (
            self.status,
            Json(ApiErrorBody {
                error: self.message,
            }),
        )
            .into_response()
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
            WorkoutError::AssociatedMesocycleNotFound { id } => {
                ApiError::not_found(format!("mesocycle {id} not found"))
            }
            WorkoutError::NotFound { id } => ApiError::not_found(format!("workout {id} not found")),
            WorkoutError::ReorderMismatch { mesocycle_id } => ApiError::unprocessable(format!(
                "reorder list does not match the workouts of mesocycle {mesocycle_id}"
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
            error @ LibraryExerciseError::SecondaryMatchesPrimary { .. } => {
                ApiError::unprocessable(error.to_string())
            }
            LibraryExerciseError::Corrupt(detail) => {
                ApiError::internal(format!("corrupt library exercise data: {detail}"))
            }
        }
    }
}

impl From<PlannedExerciseError> for ApiError {
    fn from(error: PlannedExerciseError) -> Self {
        match error {
            PlannedExerciseError::Database(error) => ApiError::internal(error),
            PlannedExerciseError::LibraryExercise(error) => error.into(),
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

impl From<SetGroupError> for ApiError {
    fn from(error: SetGroupError) -> Self {
        match error {
            SetGroupError::Database(error) => ApiError::internal(error),
            SetGroupError::AssociatedPlannedExerciseNotFound { id } => {
                ApiError::not_found(format!("planned exercise {id} not found"))
            }
            SetGroupError::AssociatedMicrocycleNotFound { id } => {
                ApiError::not_found(format!("microcycle {id} not found"))
            }
            SetGroupError::MesocycleMismatch {
                planned_exercise_id,
                microcycle_id,
            } => ApiError::unprocessable(format!(
                "microcycle {microcycle_id} is not in the same mesocycle as planned exercise {planned_exercise_id}"
            )),
            SetGroupError::NotFound { id } => {
                ApiError::not_found(format!("set group {id} not found"))
            }
            SetGroupError::ReorderMismatch {
                planned_exercise_id,
            } => ApiError::unprocessable(format!(
                "reorder list does not match the set groups of planned exercise {planned_exercise_id}"
            )),
            SetGroupError::Invalid(error) => ApiError::unprocessable(error.to_string()),
            SetGroupError::Corrupt(detail) => {
                ApiError::internal(format!("corrupt set group data: {detail}"))
            }
        }
    }
}

// An invalid rep target or intensity in a set group's prescription — an invalid
// input value, so a 422.
impl From<SetGroupTypeInputError> for ApiError {
    fn from(error: SetGroupTypeInputError) -> Self {
        ApiError::unprocessable(error.to_string())
    }
}

impl From<FullMesocycleError> for ApiError {
    fn from(error: FullMesocycleError) -> Self {
        match error {
            FullMesocycleError::Mesocycle(error) => error.into(),
            FullMesocycleError::Microcycle(error) => error.into(),
            FullMesocycleError::Workout(error) => error.into(),
            FullMesocycleError::PlannedExercise(error) => error.into(),
            FullMesocycleError::SetGroup(error) => error.into(),
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
