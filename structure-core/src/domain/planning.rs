use std::fmt::Display;

use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Name(String);

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum NameError {
    #[error("name must not be empty")]
    Empty,
}

impl Name {
    pub fn new(value: impl Into<String>) -> Result<Name, NameError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(NameError::Empty);
        }
        Ok(Name(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Mesocycle {
    id: i64,
    name: Name,
    mode: MesocycleMode,
}

impl Mesocycle {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn mode(&self) -> MesocycleMode {
        self.mode
    }

    pub(crate) fn new(id: i64, name: Name, mode: MesocycleMode) -> Mesocycle {
        Mesocycle { id, name, mode }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum MesocycleMode {
    Algorithmic,
    Manual,
}

impl Display for MesocycleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Algorithmic => "Algorithmic",
            Self::Manual => "Manual",
        };
        f.write_str(s)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Microcycle {
    id: i64,
    position: u32,
    phase: Option<Phase>,
}

impl Microcycle {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn phase(&self) -> Option<Phase> {
        self.phase
    }

    pub(crate) fn new(id: i64, position: u32, phase: Option<Phase>) -> Microcycle {
        Microcycle {
            id,
            position,
            phase,
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Accumulation,
    Intensification,
    Deload,
}

impl Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Accumulation => "Accumulation",
            Self::Intensification => "Intensification",
            Self::Deload => "Deload",
        };
        f.write_str(s)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Workout {
    id: i64,
    name: Name,
    position: u32,
}

impl Workout {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub(crate) fn new(id: i64, name: Name, position: u32) -> Workout {
        Workout { id, name, position }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PlannedExercise {
    id: i64,
    exercise: LibraryExercise,
    position: u32,
}

impl PlannedExercise {
    pub(crate) fn new(id: i64, exercise: LibraryExercise, position: u32) -> PlannedExercise {
        PlannedExercise {
            id,
            exercise,
            position,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.exercise.name()
    }

    pub fn exercise(&self) -> &LibraryExercise {
        &self.exercise
    }

    pub fn position(&self) -> u32 {
        self.position
    }
}

/// A planned rep prescription: a single count or a closed `[min, max]` range.
/// Both variants carry validated newtypes, so an invalid `RepTarget` is
/// unrepresentable even though the variants themselves are public.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum RepTarget {
    Exact(RepCount),
    Range(RepRange),
}

impl RepTarget {
    pub fn exact(reps: u32) -> Result<RepTarget, RepTargetError> {
        Ok(RepTarget::Exact(RepCount::new(reps)?))
    }

    pub fn range(min: u32, max: u32) -> Result<RepTarget, RepTargetError> {
        Ok(RepTarget::Range(RepRange::new(min, max)?))
    }
}

/// A single rep count, guaranteed to be at least 1.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct RepCount(u32);

impl RepCount {
    pub fn new(reps: u32) -> Result<RepCount, RepTargetError> {
        if reps == 0 {
            return Err(RepTargetError::ZeroReps);
        }
        Ok(RepCount(reps))
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

/// A closed rep range, guaranteed to satisfy `1 <= min < max`.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct RepRange {
    min: u32,
    max: u32,
}

impl RepRange {
    pub fn new(min: u32, max: u32) -> Result<RepRange, RepTargetError> {
        if min == 0 || max == 0 {
            return Err(RepTargetError::ZeroReps);
        }
        if max <= min {
            return Err(RepTargetError::NonAscendingRange { min, max });
        }
        Ok(RepRange { min, max })
    }

    pub fn min(self) -> u32 {
        self.min
    }

    pub fn max(self) -> u32 {
        self.max
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum RepTargetError {
    #[error("a rep target must prescribe at least one rep")]
    ZeroReps,
    #[error("rep range max {max} must be greater than min {min}")]
    NonAscendingRange { min: u32, max: u32 },
}

/// The one prescription that drives a set group, drawn from two mutually
/// exclusive families: proximity-to-failure (`Rir`/`Rpe`) or weight-resolving
/// (`PercentOneRepMax`/`TargetWeight`/`WeightIncrement`). Effort and weight are
/// never prescribed together.
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Intensity {
    Rir(Rir),
    Rpe(Rpe),
    PercentOneRepMax(PercentOneRepMax),
    TargetWeight(Weight),
    WeightIncrement(Weight),
}

/// Whether a `Load` is valid for an exercise of the given `ExerciseType`.
pub(crate) fn load_matches_exercise_type(exercise_type: ExerciseType, load: Load) -> bool {
    matches!(
        (exercise_type, load),
        (ExerciseType::Bodyweight, Load::Bodyweight)
            | (
                ExerciseType::WeightedBodyweight,
                Load::WeightedBodyweight { .. }
            )
            | (
                ExerciseType::AssistedBodyweight,
                Load::AssistedBodyweight { .. }
            )
            | (ExerciseType::Weighted, Load::Weighted { .. })
    )
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LibraryExercise {
    id: i64,
    name: Name,
    exercise_type: ExerciseType,
}

impl LibraryExercise {
    pub(crate) fn new(id: i64, name: Name, exercise_type: ExerciseType) -> LibraryExercise {
        LibraryExercise {
            id,
            name,
            exercise_type,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn exercise_type(&self) -> ExerciseType {
        self.exercise_type
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum ExerciseType {
    Bodyweight,
    WeightedBodyweight,
    AssistedBodyweight,
    Weighted,
}

impl Display for ExerciseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Bodyweight => "Bodyweight",
            Self::WeightedBodyweight => "WeightedBodyweight",
            Self::AssistedBodyweight => "AssistedBodyweight",
            Self::Weighted => "Weighted",
        };
        f.write_str(s)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SetValidationError {
    #[error("load {load:?} is incompatible with exercise type {exercise_type:?}")]
    LoadMismatch {
        load: Load,
        exercise_type: ExerciseType,
    },
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Set {
    id: i64,
    position: u32,
    load: Load,
    reps: Option<u32>,
    set_type: SetType,
}

impl Set {
    pub(crate) fn new(
        id: i64,
        position: u32,
        exercise_type: ExerciseType,
        load: Load,
        reps: Option<u32>,
        set_type: SetType,
    ) -> Result<Set, SetValidationError> {
        if !load_matches_exercise_type(exercise_type, load) {
            return Err(SetValidationError::LoadMismatch {
                load,
                exercise_type,
            });
        }
        Ok(Set {
            id,
            position,
            load,
            reps,
            set_type,
        })
    }

    /// Builds a `Set` from already-persisted columns without re-checking the
    /// load against the exercise type. Use only on the read path: the invariant
    /// was enforced by `new` on write, so re-validating would only force reads
    /// to join in the exercise type.
    pub(crate) fn new_unchecked(
        id: i64,
        position: u32,
        load: Load,
        reps: Option<u32>,
        set_type: SetType,
    ) -> Set {
        Set {
            id,
            position,
            load,
            reps,
            set_type,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn load(&self) -> Load {
        self.load
    }

    pub fn reps(&self) -> Option<u32> {
        self.reps
    }

    pub fn set_type(&self) -> SetType {
        self.set_type
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum SetType {
    Regular { effort: Option<Effort> },
    Myorep,
    MyorepMatch,
    Drop,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Load {
    Bodyweight,
    WeightedBodyweight { added_weight: Option<Weight> },
    AssistedBodyweight { assistance: Option<Weight> },
    Weighted { weight: Option<Weight> },
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Weight {
    value: f64,
    unit: WeightUnit,
}

impl Weight {
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn unit(&self) -> WeightUnit {
        self.unit
    }

    pub fn new(value: f64, unit: WeightUnit) -> Weight {
        Weight { value, unit }
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum WeightUnit {
    Kg,
    Lbs,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum EffortError {
    #[error(transparent)]
    InvalidRpe(#[from] RpeError),
    #[error(transparent)]
    InvalidRir(#[from] RirError),
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Effort {
    Rir(Rir),
    Rpe(Rpe),
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("rpe must be between 1 and 11, got {0}")]
pub struct RpeError(u8);

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Rpe(u8);

impl Rpe {
    pub fn new(value: u8) -> Result<Rpe, RpeError> {
        if !(1..=11).contains(&value) {
            return Err(RpeError(value));
        }
        Ok(Rpe(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("RIR must be between -1 and 10, got {0}")]
pub struct RirError(i8);

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Rir(i8);

impl Rir {
    pub fn new(value: i8) -> Result<Rir, RirError> {
        if !(-1..=10).contains(&value) {
            return Err(RirError(value));
        }
        Ok(Rir(value))
    }

    pub fn value(self) -> i8 {
        self.0
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("percent of 1RM must be between 1 and 100, got {0}")]
pub struct PercentOneRepMaxError(u8);

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct PercentOneRepMax(u8);

impl PercentOneRepMax {
    pub fn new(value: u8) -> Result<PercentOneRepMax, PercentOneRepMaxError> {
        if !(1..=100).contains(&value) {
            return Err(PercentOneRepMaxError(value));
        }
        Ok(PercentOneRepMax(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workout_has_correct_name_and_id_and_position() {
        let workout = Workout::new(1, Name::new("test workout").unwrap(), 0);

        assert_eq!(workout.id(), 1);
        assert_eq!(workout.name(), "test workout");
        assert_eq!(workout.position(), 0);
    }

    #[test]
    fn new_microcycle_has_correct_position_and_id_and_phase() {
        let microcycle = Microcycle::new(1234, 0, Some(Phase::Deload));

        assert_eq!(microcycle.position(), 0);
        assert_eq!(microcycle.id(), 1234);
        assert_eq!(microcycle.phase(), Some(Phase::Deload));
    }

    #[test]
    fn new_mesocycle_has_correct_name_id_and_mode() {
        let mesocycle = Mesocycle::new(
            1,
            Name::new("test mesocycle").unwrap(),
            MesocycleMode::Algorithmic,
        );

        assert_eq!(mesocycle.name(), "test mesocycle");
        assert_eq!(mesocycle.id(), 1);
        assert_eq!(mesocycle.mode(), MesocycleMode::Algorithmic);
    }

    #[test]
    fn new_bodyweight_exercise_has_bodyweight_type_with_correct_name_and_id_and_position() {
        let exercise =
            LibraryExercise::new(2, Name::new("Squat").unwrap(), ExerciseType::Bodyweight);
        let planned_exercise = PlannedExercise::new(1, exercise, 1);

        assert_eq!(
            planned_exercise.exercise().exercise_type(),
            ExerciseType::Bodyweight
        );
        assert_eq!(planned_exercise.name(), "Squat");
        assert_eq!(planned_exercise.id(), 1);
        assert_eq!(planned_exercise.position(), 1);
    }

    #[test]
    fn new_weighted_exercise_has_weighted_type_with_correct_name_and_id_and_position() {
        let exercise = LibraryExercise::new(2, Name::new("Squat").unwrap(), ExerciseType::Weighted);
        let planned_exercise = PlannedExercise::new(1, exercise, 2);

        assert_eq!(
            planned_exercise.exercise().exercise_type(),
            ExerciseType::Weighted
        );
        assert_eq!(planned_exercise.name(), "Squat");
        assert_eq!(planned_exercise.id(), 1);
        assert_eq!(planned_exercise.position(), 2);
    }

    #[test]
    fn new_assisted_bodyweight_exercise_has_assisted_bodyweight_type_with_correct_name_and_id_and_position()
     {
        let exercise = LibraryExercise::new(
            2,
            Name::new("Squat").unwrap(),
            ExerciseType::AssistedBodyweight,
        );
        let planned_exercise = PlannedExercise::new(3, exercise, 5);

        assert_eq!(
            planned_exercise.exercise().exercise_type(),
            ExerciseType::AssistedBodyweight
        );
        assert_eq!(planned_exercise.name(), "Squat");
        assert_eq!(planned_exercise.id(), 3);
        assert_eq!(planned_exercise.position(), 5);
    }

    #[test]
    fn rep_target_keeps_its_prescription() {
        assert_eq!(
            RepTarget::exact(8).unwrap(),
            RepTarget::Exact(RepCount::new(8).unwrap())
        );

        let RepTarget::Range(range) = RepTarget::range(8, 12).unwrap() else {
            panic!("range constructor should produce a Range");
        };
        assert_eq!(range.min(), 8);
        assert_eq!(range.max(), 12);
    }

    #[test]
    fn rep_target_rejects_zero_reps() {
        assert_eq!(RepTarget::exact(0), Err(RepTargetError::ZeroReps));
        assert_eq!(RepTarget::range(0, 5), Err(RepTargetError::ZeroReps));
        assert_eq!(RepTarget::range(5, 0), Err(RepTargetError::ZeroReps));
    }

    #[test]
    fn rep_target_rejects_non_ascending_range() {
        assert_eq!(
            RepTarget::range(10, 10),
            Err(RepTargetError::NonAscendingRange { min: 10, max: 10 })
        );
        assert_eq!(
            RepTarget::range(12, 8),
            Err(RepTargetError::NonAscendingRange { min: 12, max: 8 })
        );
    }

    #[test]
    fn percent_one_rep_max_enforces_its_bounds() {
        assert_eq!(PercentOneRepMax::new(0), Err(PercentOneRepMaxError(0)));
        assert_eq!(PercentOneRepMax::new(101), Err(PercentOneRepMaxError(101)));
        assert_eq!(PercentOneRepMax::new(80).unwrap().value(), 80);
    }

    #[test]
    fn new_weighted_bodyweight_exercise_has_weighted_bodyweight_type_with_correct_name_and_id() {
        let exercise = LibraryExercise::new(
            2,
            Name::new("Pull Ups").unwrap(),
            ExerciseType::WeightedBodyweight,
        );
        let planned_exercise = PlannedExercise::new(15, exercise, 9);

        assert_eq!(
            planned_exercise.exercise().exercise_type(),
            ExerciseType::WeightedBodyweight
        );
        assert_eq!(planned_exercise.name(), "Pull Ups");
        assert_eq!(planned_exercise.id(), 15);
        assert_eq!(planned_exercise.position(), 9);
    }
}
