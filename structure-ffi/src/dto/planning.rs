use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use structure_core::domain::planning::{
    ExerciseType, LibraryExercise, MesocycleMode, Microcycle, Phase, PlannedExercise, Workout,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MesocycleDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) mode: MesocycleModeDTO,
    pub(crate) microcycle_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum MesocycleModeDTO {
    Algorithmic,
    Manual,
}

impl From<MesocycleMode> for MesocycleModeDTO {
    fn from(value: MesocycleMode) -> Self {
        match value {
            MesocycleMode::Algorithmic => Self::Algorithmic,
            MesocycleMode::Manual => Self::Manual,
        }
    }
}

impl From<MesocycleModeDTO> for MesocycleMode {
    fn from(value: MesocycleModeDTO) -> Self {
        match value {
            MesocycleModeDTO::Algorithmic => Self::Algorithmic,
            MesocycleModeDTO::Manual => Self::Manual,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MicrocycleDTO {
    pub(crate) id: i64,
    pub(crate) position: u32,
    pub(crate) phase: Option<PhaseDTO>,
}

impl From<&Microcycle> for MicrocycleDTO {
    fn from(value: &Microcycle) -> Self {
        MicrocycleDTO {
            id: value.id(),
            position: value.position(),
            phase: value.phase().map(PhaseDTO::from),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum PhaseDTO {
    Accumulation,
    Intensification,
    Deload,
}

impl From<Phase> for PhaseDTO {
    fn from(value: Phase) -> Self {
        match value {
            Phase::Accumulation => PhaseDTO::Accumulation,
            Phase::Intensification => PhaseDTO::Intensification,
            Phase::Deload => PhaseDTO::Deload,
        }
    }
}

impl From<PhaseDTO> for Phase {
    fn from(value: PhaseDTO) -> Self {
        match value {
            PhaseDTO::Accumulation => Self::Accumulation,
            PhaseDTO::Intensification => Self::Intensification,
            PhaseDTO::Deload => Self::Deload,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct WorkoutDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) position: u32,
}

impl From<&Workout> for WorkoutDTO {
    fn from(value: &Workout) -> Self {
        WorkoutDTO {
            id: value.id(),
            name: value.name().to_owned(),
            position: value.position(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum ExerciseTypeDTO {
    Bodyweight,
    WeightedBodyweight,
    AssistedBodyweight,
    Weighted,
}

impl From<ExerciseType> for ExerciseTypeDTO {
    fn from(value: ExerciseType) -> Self {
        match value {
            ExerciseType::Bodyweight => ExerciseTypeDTO::Bodyweight,
            ExerciseType::WeightedBodyweight => ExerciseTypeDTO::WeightedBodyweight,
            ExerciseType::AssistedBodyweight => ExerciseTypeDTO::AssistedBodyweight,
            ExerciseType::Weighted => ExerciseTypeDTO::Weighted,
        }
    }
}

impl From<ExerciseTypeDTO> for ExerciseType {
    fn from(dto: ExerciseTypeDTO) -> Self {
        match dto {
            ExerciseTypeDTO::Bodyweight => Self::Bodyweight,
            ExerciseTypeDTO::WeightedBodyweight => Self::WeightedBodyweight,
            ExerciseTypeDTO::AssistedBodyweight => Self::AssistedBodyweight,
            ExerciseTypeDTO::Weighted => Self::Weighted,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[frb]
pub struct PlannedExerciseDTO {
    pub(crate) id: i64,
    pub(crate) exercise: LibraryExerciseDTO,
    pub(crate) position: u32,
}

impl From<&PlannedExercise> for PlannedExerciseDTO {
    fn from(value: &PlannedExercise) -> Self {
        PlannedExerciseDTO {
            id: value.id(),
            exercise: LibraryExerciseDTO::from(value.exercise()),
            position: value.position(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[frb]
pub struct LibraryExerciseDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) exercise_type: ExerciseTypeDTO,
}

impl From<&LibraryExercise> for LibraryExerciseDTO {
    fn from(value: &LibraryExercise) -> Self {
        LibraryExerciseDTO {
            id: value.id(),
            name: value.name().to_owned(),
            exercise_type: ExerciseTypeDTO::from(value.exercise_type()),
        }
    }
}
