use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Mesocycle {
    id: i64,
    name: String,
}

impl Mesocycle {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn new(id: i64, name: impl Into<String>) -> Mesocycle {
        Mesocycle {
            id,
            name: name.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ExerciseType {
    Bodyweight,
    WeightedBodyweight,
    AssistedBodyweight,
    Weighted,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Exercise {
    id: i64,
    name: String,
    exercise_type: ExerciseType,
    sets: Vec<Set>,
}

impl Exercise {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sets(&self) -> &[Set] {
        &self.sets
    }

    pub fn exercise_type(&self) -> ExerciseType {
        self.exercise_type
    }

    pub(crate) fn bodyweight(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Bodyweight,
        }
    }

    pub(crate) fn weighted(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Weighted,
        }
    }

    pub(crate) fn assisted_bodyweight(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::AssistedBodyweight,
        }
    }

    pub(crate) fn weighted_bodyweight(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::WeightedBodyweight,
        }
    }

    pub fn add_set(&mut self, set: Set) -> Result<(), String> {
        match (&self.exercise_type, &set.load()) {
            (ExerciseType::Bodyweight, Load::Bodyweight)
            | (ExerciseType::WeightedBodyweight, Load::WeightedBodyweight { .. })
            | (ExerciseType::AssistedBodyweight, Load::AssistedBodyweight { .. })
            | (ExerciseType::Weighted, Load::Weighted { .. }) => {
                self.sets.push(set);
                Ok(())
            }
            (exercise, set) => Err(format!(
                "Cannot add set with load {:?} to {:?} exercise type",
                set, exercise
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Set {
    Regular {
        load: Load,
        reps: Option<u32>,
        effort: Option<Effort>,
    },
    Myorep {
        load: Load,
        reps: Option<u32>,
    },
    MyorepMatch {
        load: Load,
        reps: Option<u32>,
    },
    Drop {
        load: Load,
        reps: Option<u32>,
        effort: Option<Effort>,
    },
}

impl Set {
    pub fn load(&self) -> &Load {
        match self {
            Set::Regular { load, .. }
            | Set::Myorep { load, .. }
            | Set::MyorepMatch { load, .. }
            | Set::Drop { load, .. } => load,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Load {
    Bodyweight,
    WeightedBodyweight { added_weight: Option<Weight> },
    AssistedBodyweight { assistance: Option<Weight> },
    Weighted { weight: Option<Weight> },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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

    pub(crate) fn new(value: f64, unit: WeightUnit) -> Weight {
        Weight { value, unit }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum WeightUnit {
    Kg,
    Lbs,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Effort {
    Rir(Rir),
    Rpe(Rpe),
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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
    fn new_mesocycle_has_correct_name_and_id() {
        let mesocycle = Mesocycle::new(1, "test mesocycle");

        assert_eq!(mesocycle.name(), "test mesocycle");
        assert_eq!(mesocycle.id(), 1)
    }

    #[test]
    fn new_bodyweight_exercise_has_bodyweight_type_with_correct_name_and_id() {
        let exercise = Exercise::bodyweight(1, "Squat");

        assert_eq!(exercise.exercise_type(), ExerciseType::Bodyweight);
        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.id(), 1);
    }

    #[test]
    fn new_weighted_exercise_has_weighted_type_with_correct_name_and_id() {
        let exercise = Exercise::weighted(1, "Squat");

        assert_eq!(exercise.exercise_type(), ExerciseType::Weighted);
        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.id(), 1);
    }

    #[test]
    fn new_assisted_bodyweight_exercise_has_assisted_bodyweight_type_with_correct_name_and_id() {
        let exercise = Exercise::assisted_bodyweight(1, "Squat");

        assert_eq!(exercise.exercise_type(), ExerciseType::AssistedBodyweight);
        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.id(), 1);
    }

    #[test]
    fn new_weighted_bodyweight_exercise_has_weighted_bodyweight_type_with_correct_name_and_id() {
        let exercise = Exercise::weighted_bodyweight(1, "Pull Ups");

        assert_eq!(exercise.exercise_type(), ExerciseType::WeightedBodyweight);
        assert_eq!(exercise.name(), "Pull Ups");
        assert_eq!(exercise.id(), 1);
    }

    #[test]
    fn add_set_to_exercise_with_matching_types_works() {
        let mut exercise = Exercise::bodyweight(1, "Bench Press");
        assert_eq!(exercise.sets().len(), 0);

        let set = Set::Regular {
            load: Load::Bodyweight,
            reps: None,
            effort: None,
        };

        exercise
            .add_set(set)
            .expect("adding matching set type should succeed");

        assert_eq!(exercise.sets().len(), 1);
        assert_eq!(
            exercise.sets()[0],
            Set::Regular {
                load: Load::Bodyweight,
                reps: None,
                effort: None
            }
        );
    }

    #[test]
    fn add_set_to_exercise_with_mismatching_types_causes_error() {
        let mut exercise = Exercise::weighted(1, "Bench Press");

        let set = Set::Regular {
            load: Load::Bodyweight,
            reps: None,
            effort: None,
        };

        let result = exercise.add_set(set);

        assert!(
            result.is_err(),
            "Should return error when set type doesn't match exercise type"
        )
    }
}
