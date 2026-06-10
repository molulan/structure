use std::fmt::Display;

use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Mesocycle {
    id: i64,
    name: String,
    mode: MesocycleMode,
}

impl Mesocycle {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mode(&self) -> MesocycleMode {
        self.mode
    }

    pub(crate) fn new(id: i64, name: impl Into<String>, mode: MesocycleMode) -> Mesocycle {
        Mesocycle {
            id,
            name: name.into(),
            mode,
        }
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
}

impl Microcycle {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub(crate) fn new(id: i64, position: u32) -> Microcycle {
        Microcycle { id, position }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Workout {
    id: i64,
    name: String,
    position: u32,
}

impl Workout {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub(crate) fn new(id: i64, name: impl Into<String>, position: u32) -> Workout {
        Workout {
            id,
            name: name.into(),
            position,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct PlannedExercise {
    id: i64,
    exercise: Exercise,
    position: u32,
    sets: Vec<Set>,
}

#[derive(Debug, thiserror::Error)]
pub enum PlannedExerciseValidationError {
    #[error("load {load:?} is incompatible with exercise type {exercise_type:?}")]
    LoadMismatch {
        load: Load,
        exercise_type: ExerciseType,
    },
}

impl PlannedExercise {
    pub(crate) fn new(
        id: i64,
        exercise: Exercise,
        position: u32,
        sets: Vec<Set>,
    ) -> Result<PlannedExercise, PlannedExerciseValidationError> {
        for set in &sets {
            if !load_matches_exercise_type(exercise.exercise_type(), set.load()) {
                return Err(PlannedExerciseValidationError::LoadMismatch {
                    load: set.load(),
                    exercise_type: exercise.exercise_type(),
                });
            }
        }

        Ok(PlannedExercise {
            id,
            exercise,
            position,
            sets,
        })
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.exercise.name()
    }

    pub fn exercise(&self) -> &Exercise {
        &self.exercise
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn sets(&self) -> &[Set] {
        &self.sets
    }

    pub fn add_set(&mut self, set: Set) -> Result<(), PlannedExerciseValidationError> {
        if !load_matches_exercise_type(self.exercise.exercise_type, set.load()) {
            return Err(PlannedExerciseValidationError::LoadMismatch {
                load: set.load(),
                exercise_type: self.exercise.exercise_type(),
            });
        }
        self.sets.push(set);
        Ok(())
    }
}

fn load_matches_exercise_type(exercise_type: ExerciseType, load: Load) -> bool {
    //should they still be references even though they are copy?
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
pub struct Exercise {
    id: i64,
    name: String,
    exercise_type: ExerciseType,
}

impl Exercise {
    pub(crate) fn new(id: i64, name: impl Into<String>, exercise_type: ExerciseType) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            exercise_type,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
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
        load: Load,
        reps: Option<u32>,
        set_type: SetType,
    ) -> Self {
        Self {
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
    Drop { effort: Option<Effort> },
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

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum Effort {
    Rir(Rir),
    Rpe(Rpe),
}
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Rpe(u8);

impl Rpe {
    pub fn new(value: u8) -> Result<Rpe, String> {
        if !(1..=11).contains(&value) {
            return Err(String::from("rpe must be between 1 and 11"));
        }
        Ok(Rpe(value))
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Rir(i8);

impl Rir {
    pub fn new(value: i8) -> Result<Rir, String> {
        if !(-1..=10).contains(&value) {
            return Err(String::from("RIR must be between -1 and 10"));
        }
        Ok(Rir(value))
    }

    pub fn value(self) -> i8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workout_has_correct_name_and_id_and_position() {
        let workout = Workout::new(1, "test workout", 0);

        assert_eq!(workout.id(), 1);
        assert_eq!(workout.name(), "test workout");
        assert_eq!(workout.position(), 0);
    }

    #[test]
    fn new_microcycle_has_correct_position_and_id() {
        let microcycle = Microcycle::new(1234, 0);

        assert_eq!(microcycle.position(), 0);
        assert_eq!(microcycle.id(), 1234);
    }

    #[test]
    fn new_mesocycle_has_correct_name_id_and_mode() {
        let mesocycle = Mesocycle::new(1, "test mesocycle", MesocycleMode::Algorithmic);

        assert_eq!(mesocycle.name(), "test mesocycle");
        assert_eq!(mesocycle.id(), 1);
        assert_eq!(mesocycle.mode(), MesocycleMode::Algorithmic);
    }

    #[test]
    fn new_bodyweight_exercise_has_bodyweight_type_with_correct_name_and_id_and_position() {
        let exercise = Exercise::new(2, "Squat", ExerciseType::Bodyweight);
        let planned_exercise = PlannedExercise::new(1, exercise, 1, vec![])
            .expect("newly created exercise has no sets, so validation cannot fail");

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
        let exercise = Exercise::new(2, "Squat", ExerciseType::Weighted);
        let planned_exercise = PlannedExercise::new(1, exercise, 2, vec![])
            .expect("newly created exercise has no sets, so validation cannot fail");

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
        let exercise = Exercise::new(2, "Squat", ExerciseType::AssistedBodyweight);
        let planned_exercise = PlannedExercise::new(3, exercise, 5, vec![])
            .expect("newly created exercise has no sets, so validation cannot fail");

        assert_eq!(
            planned_exercise.exercise().exercise_type(),
            ExerciseType::AssistedBodyweight
        );
        assert_eq!(planned_exercise.name(), "Squat");
        assert_eq!(planned_exercise.id(), 3);
        assert_eq!(planned_exercise.position(), 5);
    }

    #[test]
    fn new_weighted_bodyweight_exercise_has_weighted_bodyweight_type_with_correct_name_and_id() {
        let exercise = Exercise::new(2, "Pull Ups", ExerciseType::WeightedBodyweight);
        let planned_exercise = PlannedExercise::new(15, exercise, 9, vec![])
            .expect("newly created exercise has no sets, so validation cannot fail");

        assert_eq!(
            planned_exercise.exercise().exercise_type(),
            ExerciseType::WeightedBodyweight
        );
        assert_eq!(planned_exercise.name(), "Pull Ups");
        assert_eq!(planned_exercise.id(), 15);
        assert_eq!(planned_exercise.position(), 9);
    }

    #[test]
    fn add_set_to_exercise_with_matching_types_works() {
        let exercise = Exercise::new(2, "Squat", ExerciseType::Bodyweight);
        let mut planned_exercise = PlannedExercise::new(1, exercise, 1, vec![])
            .expect("newly created exercise has no sets, so validation cannot fail");
        assert_eq!(planned_exercise.sets().len(), 0);

        let set = Set::new(
            1,
            1,
            Load::Bodyweight,
            None,
            SetType::Regular { effort: None },
        );

        planned_exercise
            .add_set(set)
            .expect("adding matching set type should succeed");

        assert_eq!(planned_exercise.sets().len(), 1);
        assert_eq!(planned_exercise.sets()[0], set);
    }

    #[test]
    fn add_set_to_exercise_with_mismatching_types_causes_error() {
        let exercise = Exercise::new(2, "Bench Press", ExerciseType::Weighted);
        let mut planned_exercise = PlannedExercise::new(1, exercise, 1, vec![])
            .expect("newly created exercise has no sets, so validation cannot fail");

        let set = Set::new(
            1,
            1,
            Load::Bodyweight,
            None,
            SetType::Regular { effort: None },
        );

        let result = planned_exercise.add_set(set);

        assert!(
            result.is_err(),
            "Should return error when set type doesn't match exercise type"
        )
    }

    #[test]
    fn new_with_single_valid_set_returns_ok_and_set_is_in_sets() {
        let exercise = Exercise::new(1, "Pull Up", ExerciseType::Bodyweight);
        let set = Set::new(
            1,
            1,
            Load::Bodyweight,
            Some(10),
            SetType::Regular { effort: None },
        );

        let planned_exercise = PlannedExercise::new(1, exercise, 0, vec![set])
            .expect("set load matches exercise type");

        assert_eq!(planned_exercise.sets().len(), 1);
        assert_eq!(planned_exercise.sets()[0], set);
    }

    #[test]
    fn new_with_multiple_valid_sets_returns_ok_with_all_sets_present() {
        let exercise = Exercise::new(1, "Pull Up", ExerciseType::Bodyweight);
        let sets = vec![
            Set::new(
                1,
                1,
                Load::Bodyweight,
                Some(10),
                SetType::Regular { effort: None },
            ),
            Set::new(2, 2, Load::Bodyweight, Some(8), SetType::Myorep),
            Set::new(3, 3, Load::Bodyweight, None, SetType::MyorepMatch),
        ];

        let planned_exercise = PlannedExercise::new(1, exercise, 0, sets.clone())
            .expect("all set loads match exercise type");

        assert_eq!(planned_exercise.sets().len(), 3);
        assert_eq!(planned_exercise.sets(), sets.as_slice());
    }

    #[test]
    fn new_with_single_mismatching_set_returns_err() {
        let exercise = Exercise::new(1, "Bench Press", ExerciseType::Weighted);
        let set = Set::new(
            1,
            1,
            Load::Bodyweight,
            Some(10),
            SetType::Regular { effort: None },
        );

        let result = PlannedExercise::new(1, exercise, 0, vec![set]);

        assert!(result.is_err(), "set load does not match exercise type");
    }

    #[test]
    fn new_with_mixed_sets_where_one_mismatches_returns_err() {
        let exercise = Exercise::new(1, "Bench Press", ExerciseType::Weighted);
        let sets = vec![
            Set::new(
                1,
                1,
                Load::Weighted { weight: None },
                Some(5),
                SetType::Regular { effort: None },
            ),
            Set::new(
                2,
                2,
                Load::Bodyweight,
                Some(10),
                SetType::Regular { effort: None },
            ),
        ];

        let result = PlannedExercise::new(1, exercise, 0, sets);

        assert!(
            result.is_err(),
            "second set load does not match exercise type"
        );
    }
}
