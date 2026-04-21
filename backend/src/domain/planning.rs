use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        Microcycle {
            id,
            position,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workout {
    id: i64,
    name: String,
}

impl Workout {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new(id: i64, name: impl Into<String>) -> Workout {
        Workout {
            id,
            name: name.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ExerciseType {
    Bodyweight,
    Weighted,
    Assisted,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Exercise {
    id: i64,
    name: String,
    sets: Vec<Set>,
    exercise_type: ExerciseType,
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

    pub fn bodyweight(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Bodyweight,
        }
    }

    pub fn weighted(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Weighted,
        }
    }

    pub fn assisted(id: i64, name: impl Into<String>) -> Exercise {
        Exercise {
            id,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Assisted,
        }
    }

    pub fn add_set(&mut self, set: Set) -> Result<(), String> {
        match (&self.exercise_type, &set) {
            (ExerciseType::Bodyweight, Set::Bodyweight { .. })
            | (ExerciseType::Assisted, Set::Assisted { .. })
            | (ExerciseType::Weighted, Set::Weighted { .. }) => {
                self.sets.push(set);
                Ok(())
            }
            (exercise, set) => Err(format!("Cannot add {:?}set to {:?}exercise", set, exercise)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Set {
    Bodyweight { reps: u32 },
    Weighted { reps: u32, weight: Weight },
    Assisted { reps: u32, assistance: Weight },
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
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum WeightUnit {
    Kg,
    Lbs,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workout_has_correct_name_and_id() {
        let workout = Workout::new(1, "test workout");

        assert_eq!(workout.name(), "test workout");
        assert_eq!(workout.id(), 1);
    }

    #[test]
    fn new_microcycle_has_correct_name_and_id() {
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
    fn new_assisted_exercise_has_assisted_type_with_correct_name_and_id() {
        let exercise = Exercise::assisted(1, "Squat");

        assert_eq!(exercise.exercise_type(), ExerciseType::Assisted);
        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.id(), 1);
    }

    #[test]
    fn add_set_to_exercise_with_matching_types_works() {
        let mut exercise = Exercise::bodyweight(1, "Bench Press");
        assert_eq!(exercise.sets().len(), 0);

        exercise.add_set(Set::Bodyweight { reps: 42 }).unwrap();

        assert_eq!(exercise.sets().len(), 1);
        assert_eq!(exercise.sets()[0], Set::Bodyweight { reps: 42 });
    }

    #[test]
    fn add_set_to_exercise_with_mismatching_types_causes_error() {
        let mut exercise = Exercise::weighted(1, "Bench Press");

        let result = exercise.add_set(Set::Bodyweight { reps: 42 });

        assert!(
            result.is_err(),
            "Should return error when set type doesn't match exercise type"
        )
    }
}
