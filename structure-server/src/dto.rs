use serde::Deserialize;
use structure_core::domain::planning::{
    Effort, EffortError, ExerciseType, Intensity, Load, MesocycleMode, PercentOneRepMax,
    PercentOneRepMaxError, Phase, PrescribedSetType, RepTarget, RepTargetError, Rir, RirError, Rpe,
    RpeError, SetGroupType, SetType, Weight, WeightUnit,
};

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

/// Input enum mirroring [`Phase`].
#[derive(Deserialize, Clone, Copy)]
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

#[derive(Deserialize, Clone, Copy)]
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

#[derive(Deserialize, Clone, Copy)]
pub struct WeightInput {
    pub value: f64,
    pub unit: WeightUnitInput,
}

impl From<WeightInput> for Weight {
    fn from(value: WeightInput) -> Self {
        Weight::new(value.value, value.unit.into())
    }
}

#[derive(Deserialize, Clone, Copy)]
pub enum LoadInput {
    Bodyweight,
    WeightedBodyweight { added_weight: Option<WeightInput> },
    AssistedBodyweight { assistance: Option<WeightInput> },
    Weighted { weight: Option<WeightInput> },
}

impl From<LoadInput> for Load {
    fn from(value: LoadInput) -> Self {
        match value {
            LoadInput::Bodyweight => Load::Bodyweight,
            LoadInput::WeightedBodyweight { added_weight } => Load::WeightedBodyweight {
                added_weight: added_weight.map(Weight::from),
            },
            LoadInput::AssistedBodyweight { assistance } => Load::AssistedBodyweight {
                assistance: assistance.map(Weight::from),
            },
            LoadInput::Weighted { weight } => Load::Weighted {
                weight: weight.map(Weight::from),
            },
        }
    }
}

/// The validated value types live here: `Rpe`/`Rir` ranges are enforced by the
/// domain constructors, so this conversion is fallible and surfaces as a 422.
#[derive(Deserialize, Clone, Copy)]
pub enum EffortInput {
    Rir(i8),
    Rpe(u8),
}

impl TryFrom<EffortInput> for Effort {
    type Error = EffortError;

    fn try_from(value: EffortInput) -> Result<Self, EffortError> {
        match value {
            EffortInput::Rir(value) => Ok(Effort::Rir(Rir::new(value)?)),
            EffortInput::Rpe(value) => Ok(Effort::Rpe(Rpe::new(value)?)),
        }
    }
}

#[derive(Deserialize, Clone, Copy)]
pub enum SetTypeInput {
    Regular { effort: Option<EffortInput> },
    Myorep,
    MyorepMatch,
    Drop,
}

impl TryFrom<SetTypeInput> for SetType {
    type Error = EffortError;

    fn try_from(value: SetTypeInput) -> Result<Self, Self::Error> {
        let effort = |effort: Option<EffortInput>| effort.map(Effort::try_from).transpose();
        Ok(match value {
            SetTypeInput::Regular { effort: e } => SetType::Regular { effort: effort(e)? },
            SetTypeInput::Myorep => SetType::Myorep,
            SetTypeInput::MyorepMatch => SetType::MyorepMatch,
            SetTypeInput::Drop => SetType::Drop,
        })
    }
}

#[derive(Deserialize)]
pub struct SetRequest {
    pub load: LoadInput,
    pub reps: Option<u32>,
    pub set_type: SetTypeInput,
}

/// The value types a set group builds from are validated by their domain
/// constructors (rep counts, `Rir`/`Rpe`/`PercentOneRepMax` ranges), so
/// assembling a [`SetGroupType`] from input is fallible and surfaces as a 422.
#[derive(Debug, thiserror::Error)]
pub enum SetGroupInputError {
    #[error(transparent)]
    RepTarget(#[from] RepTargetError),
    #[error(transparent)]
    Rir(#[from] RirError),
    #[error(transparent)]
    Rpe(#[from] RpeError),
    #[error(transparent)]
    PercentOneRepMax(#[from] PercentOneRepMaxError),
}

/// Input enum mirroring [`RepTarget`].
#[derive(Deserialize, Clone, Copy)]
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
#[derive(Deserialize, Clone, Copy)]
pub enum IntensityInput {
    Rir(i8),
    Rpe(u8),
    PercentOneRepMax(u8),
    TargetWeight(WeightInput),
    WeightIncrement(WeightInput),
}

impl TryFrom<IntensityInput> for Intensity {
    type Error = SetGroupInputError;

    fn try_from(value: IntensityInput) -> Result<Self, SetGroupInputError> {
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
#[derive(Deserialize, Clone, Copy)]
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
    type Error = SetGroupInputError;

    fn try_from(value: SetGroupTypeInput) -> Result<Self, SetGroupInputError> {
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
