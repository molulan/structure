use serde::Deserialize;
use structure_core::domain::planning::{
    ExerciseType, Intensity, IntensityError, MesocycleMode, MuscleGroup, PercentOneRepMax, Phase,
    PrescribedSetType, RepTarget, RepTargetError, Rir, Rpe, SetGroupType, Weight, WeightUnit,
};
use ts_rs::TS;

/// Input enum mirroring [`MesocycleMode`].
#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
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

#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct CreateMesocycleRequest {
    pub name: String,
    pub mode: MesocycleModeInput,
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct UpdateMesocycleRequest {
    pub name: String,
    pub mode: MesocycleModeInput,
}

/// The desired ordering of a parent's children, by id. Shared by every reorder
/// endpoint.
#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct ReorderRequest {
    pub ordered_ids: Vec<i64>,
}

/// Input enum mirroring [`Phase`].
#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
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
#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct UpdatePhaseRequest {
    pub phase: Option<PhaseInput>,
}

/// A workout's name, used for both creating and renaming.
#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct WorkoutNameRequest {
    pub name: String,
}

/// Input enum mirroring [`ExerciseType`].
#[derive(Deserialize, TS, Clone, Copy)]
#[ts(export_to = "requests.ts")]
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

/// Input enum mirroring [`MuscleGroup`].
#[derive(Deserialize, TS, Clone, Copy)]
#[ts(export_to = "requests.ts")]
pub enum MuscleGroupInput {
    Chest,
    Back,
    Traps,
    Shoulders,
    Quads,
    Hamstrings,
    Glutes,
    Biceps,
    Triceps,
    Calves,
}

impl From<MuscleGroupInput> for MuscleGroup {
    fn from(value: MuscleGroupInput) -> Self {
        match value {
            MuscleGroupInput::Chest => MuscleGroup::Chest,
            MuscleGroupInput::Back => MuscleGroup::Back,
            MuscleGroupInput::Traps => MuscleGroup::Traps,
            MuscleGroupInput::Shoulders => MuscleGroup::Shoulders,
            MuscleGroupInput::Quads => MuscleGroup::Quads,
            MuscleGroupInput::Hamstrings => MuscleGroup::Hamstrings,
            MuscleGroupInput::Glutes => MuscleGroup::Glutes,
            MuscleGroupInput::Biceps => MuscleGroup::Biceps,
            MuscleGroupInput::Triceps => MuscleGroup::Triceps,
            MuscleGroupInput::Calves => MuscleGroup::Calves,
        }
    }
}

/// A library exercise's fields, used for both creating and updating. Secondary
/// muscle groups default to empty when the field is omitted.
#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct LibraryExerciseRequest {
    pub name: String,
    pub exercise_type: ExerciseTypeInput,
    pub primary_muscle_group: MuscleGroupInput,
    // `serde(default)` is invisible to ts-rs, so the generated type is told
    // separately that this field may be omitted.
    #[serde(default)]
    #[ts(as = "Option<_>", optional)]
    pub secondary_muscle_groups: Vec<MuscleGroupInput>,
}

impl LibraryExerciseRequest {
    /// The secondary muscle groups as domain values, ready to hand to persistence.
    pub fn secondary_muscle_groups(&self) -> Vec<MuscleGroup> {
        self.secondary_muscle_groups
            .iter()
            .map(|&m| m.into())
            .collect()
    }
}

/// Which library exercise to place into a workout.
#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
pub struct PlannedExerciseRequest {
    pub library_exercise_id: i64,
}

#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
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

#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
pub struct WeightInput {
    pub value: f64,
    pub unit: WeightUnitInput,
}

impl From<WeightInput> for Weight {
    fn from(value: WeightInput) -> Self {
        Weight::new(value.value, value.unit.into())
    }
}

/// Input enum mirroring [`RepTarget`].
#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
pub enum RepTargetInput {
    Exact(u32),
    Range { min: u32, max: u32 },
    AtLeast(u32),
}

impl TryFrom<RepTargetInput> for RepTarget {
    type Error = RepTargetError;

    fn try_from(value: RepTargetInput) -> Result<Self, RepTargetError> {
        match value {
            RepTargetInput::Exact(reps) => RepTarget::exact(reps),
            RepTargetInput::Range { min, max } => RepTarget::range(min, max),
            RepTargetInput::AtLeast(reps) => RepTarget::at_least(reps),
        }
    }
}

/// Input enum mirroring [`Intensity`].
#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
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
#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
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
#[derive(Deserialize, TS)]
#[ts(export_to = "requests.ts")]
pub enum SetGroupTypeInput {
    Prescribed {
        set_type: PrescribedSetTypeInput,
        reps: RepTargetInput,
        intensity: IntensityInput,
    },
    MyorepMatch,
}

/// Assembling a [`SetGroupType::Prescribed`] from input fails in two independent
/// ways — the rep target and the intensity — each validated by its own domain
/// constructor; this unions them into the conversion's single error type.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SetGroupTypeInputError {
    #[error(transparent)]
    RepTarget(#[from] RepTargetError),
    #[error(transparent)]
    Intensity(#[from] IntensityError),
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

#[derive(Deserialize, TS)]
#[ts(export, export_to = "requests.ts")]
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
