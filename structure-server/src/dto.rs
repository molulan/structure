use serde::Deserialize;
use structure_core::domain::planning::{ExerciseType, MesocycleMode};

/// Input enum mirroring [`MesocycleMode`].
#[derive(Deserialize, Clone, Copy)]
pub enum MesocycleModeInput {
    Algorithmic,
    Manual,
}

impl From<MesocycleModeInput> for MesocycleMode {
    fn from(value: MesocycleModeInput) -> Self {
        match value {
            MesocycleModeInput::Algorithmic => MesocycleMode::Algorithmic,
            MesocycleModeInput::Manual => MesocycleMode::Manual,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateMesocycleRequest {
    pub name: String,
    pub mode: MesocycleModeInput,
}

#[derive(Deserialize)]
pub struct UpdateMesocycleRequest {
    pub name: String,
    pub mode: MesocycleModeInput,
}

/// The desired ordering of a parent's children, by id. Shared by every reorder
/// endpoint.
#[derive(Deserialize)]
pub struct ReorderRequest {
    pub ordered_ids: Vec<i64>,
}

/// A workout's name, used for both creating and renaming.
#[derive(Deserialize)]
pub struct WorkoutNameRequest {
    pub name: String,
}

/// Input enum mirroring [`ExerciseType`].
#[derive(Deserialize, Clone, Copy)]
pub enum ExerciseTypeInput {
    Bodyweight,
    WeightedBodyweight,
    AssistedBodyweight,
    Weighted,
}

impl From<ExerciseTypeInput> for ExerciseType {
    fn from(value: ExerciseTypeInput) -> Self {
        match value {
            ExerciseTypeInput::Bodyweight => ExerciseType::Bodyweight,
            ExerciseTypeInput::WeightedBodyweight => ExerciseType::WeightedBodyweight,
            ExerciseTypeInput::AssistedBodyweight => ExerciseType::AssistedBodyweight,
            ExerciseTypeInput::Weighted => ExerciseType::Weighted,
        }
    }
}

/// A library exercise's fields, used for both creating and updating.
#[derive(Deserialize)]
pub struct LibraryExerciseRequest {
    pub name: String,
    pub exercise_type: ExerciseTypeInput,
}

/// Which library exercise to place into a workout.
#[derive(Deserialize)]
pub struct PlannedExerciseRequest {
    pub library_exercise_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesocycle_mode_input_converts_to_domain() {
        assert_eq!(
            MesocycleMode::from(MesocycleModeInput::Algorithmic),
            MesocycleMode::Algorithmic
        );
        assert_eq!(
            MesocycleMode::from(MesocycleModeInput::Manual),
            MesocycleMode::Manual
        );
    }
}
