use serde::Deserialize;
use structure_core::domain::planning::{
    ExerciseType, Intensity, IntensityError, MesocycleMode, PercentOneRepMax, Phase,
    PrescribedSetType, RepTarget, RepTargetError, Rir, Rpe, SetGroupType, Weight, WeightUnit,
};

/// Input enum mirroring [`MesocycleMode`].
#[derive(Deserialize)]
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

/// Input enum mirroring [`Phase`].
#[derive(Deserialize)]
pub enum PhaseInput {
    Accumulation,
    Intensification,
    Deload,
}

impl From<PhaseInput> for Phase {
    fn from(value: PhaseInput) -> Self {
        match value {
            PhaseInput::Accumulation => Phase::Accumulation,
            PhaseInput::Intensification => Phase::Intensification,
            PhaseInput::Deload => Phase::Deload,
        }
    }
}

/// A microcycle's phase. `null` clears it.
#[derive(Deserialize)]
pub struct UpdatePhaseRequest {
    pub phase: Option<PhaseInput>,
}

/// A workout's name, used for both creating and renaming.
#[derive(Deserialize)]
pub struct WorkoutNameRequest {
    pub name: String,
}

/// Input enum mirroring [`ExerciseType`].
#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub enum WeightUnitInput {
    Kg,
    Lbs,
}

impl From<WeightUnitInput> for WeightUnit {
    fn from(value: WeightUnitInput) -> Self {
        match value {
            WeightUnitInput::Kg => WeightUnit::Kg,
            WeightUnitInput::Lbs => WeightUnit::Lbs,
        }
    }
}

#[derive(Deserialize)]
pub struct WeightInput {
    pub value: f64,
    pub unit: WeightUnitInput,
}

impl From<WeightInput> for Weight {
    fn from(value: WeightInput) -> Self {
        Weight::new(value.value, value.unit.into())
    }
}

/// Assembling a [`SetGroupType::Prescribed`] from input is fallible in two
/// independent ways — the rep target and the intensity — each validated by its
/// domain constructor. This unions them so the conversion surfaces a 422.
#[derive(Debug, thiserror::Error)]
pub enum SetGroupTypeInputError {
    #[error(transparent)]
    RepTarget(#[from] RepTargetError),
    #[error(transparent)]
    Intensity(#[from] IntensityError),
}

/// Input enum mirroring [`RepTarget`].
#[derive(Deserialize)]
pub enum RepTargetInput {
    Exact(u32),
    Range { min: u32, max: u32 },
}

impl TryFrom<RepTargetInput> for RepTarget {
    type Error = RepTargetError;

    fn try_from(value: RepTargetInput) -> Result<Self, RepTargetError> {
        match value {
            RepTargetInput::Exact(reps) => RepTarget::exact(reps),
            RepTargetInput::Range { min, max } => RepTarget::range(min, max),
        }
    }
}

/// Input enum mirroring [`Intensity`].
#[derive(Deserialize)]
pub enum IntensityInput {
    Rir(i8),
    Rpe(u8),
    PercentOneRepMax(u8),
    TargetWeight(WeightInput),
    WeightIncrement(WeightInput),
}

impl TryFrom<IntensityInput> for Intensity {
    type Error = IntensityError;

    fn try_from(value: IntensityInput) -> Result<Self, IntensityError> {
        Ok(match value {
            IntensityInput::Rir(value) => Intensity::Rir(Rir::new(value)?),
            IntensityInput::Rpe(value) => Intensity::Rpe(Rpe::new(value)?),
            IntensityInput::PercentOneRepMax(value) => {
                Intensity::PercentOneRepMax(PercentOneRepMax::new(value)?)
            }
            IntensityInput::TargetWeight(weight) => Intensity::TargetWeight(weight.into()),
            IntensityInput::WeightIncrement(weight) => Intensity::WeightIncrement(weight.into()),
        })
    }
}

/// Input enum mirroring [`PrescribedSetType`].
#[derive(Deserialize)]
pub enum PrescribedSetTypeInput {
    Regular,
    Myorep,
    Drop,
}

impl From<PrescribedSetTypeInput> for PrescribedSetType {
    fn from(value: PrescribedSetTypeInput) -> Self {
        match value {
            PrescribedSetTypeInput::Regular => PrescribedSetType::Regular,
            PrescribedSetTypeInput::Myorep => PrescribedSetType::Myorep,
            PrescribedSetTypeInput::Drop => PrescribedSetType::Drop,
        }
    }
}

/// Input enum mirroring [`SetGroupType`].
#[derive(Deserialize)]
pub enum SetGroupTypeInput {
    Prescribed {
        set_type: PrescribedSetTypeInput,
        reps: RepTargetInput,
        intensity: IntensityInput,
    },
    MyorepMatch,
}

impl TryFrom<SetGroupTypeInput> for SetGroupType {
    type Error = SetGroupTypeInputError;

    fn try_from(value: SetGroupTypeInput) -> Result<Self, SetGroupTypeInputError> {
        Ok(match value {
            SetGroupTypeInput::Prescribed {
                set_type,
                reps,
                intensity,
            } => SetGroupType::Prescribed {
                set_type: set_type.into(),
                reps: RepTarget::try_from(reps)?,
                intensity: Intensity::try_from(intensity)?,
            },
            SetGroupTypeInput::MyorepMatch => SetGroupType::MyorepMatch,
        })
    }
}

#[derive(Deserialize)]
pub struct SetGroupRequest {
    pub number_of_sets: u32,
    pub set_group_type: SetGroupTypeInput,
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
