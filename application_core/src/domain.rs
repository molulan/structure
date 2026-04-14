use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum WeightUnit {
    Kg,
    Lbs,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct Mesocycle {
    pub id: Option<i64>,
    pub name: String,
    pub microcycles: Vec<Microcycle>,
}

impl Mesocycle {
    pub fn new(name: impl Into<String>) -> Mesocycle {
        Mesocycle {
            id: None,
            name: name.into(),
            microcycles: vec![],
        }
    }

    pub fn add_microcycle(&mut self, microcycle: Microcycle) {
        self.microcycles.push(microcycle);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct Microcycle {
    pub id: Option<i64>,
    pub name: String,
    pub workouts: Vec<Workout>,
}

impl Microcycle {
    pub fn new(name: impl Into<String>) -> Microcycle {
        Microcycle {
            id: None,
            name: name.into(),
            workouts: vec![],
        }
    }

    pub fn add_workout(&mut self, workout: Workout) {
        self.workouts.push(workout);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct Workout {
    pub id: Option<i64>,
    pub name: String,
    pub exercises: Vec<Exercise>,
}

impl Workout {
    pub fn new(name: impl Into<String>) -> Workout {
        Workout {
            id: None,
            name: name.into(),
            exercises: vec![],
        }
    }

    pub fn add_exercise(&mut self, exercise: Exercise) {
        self.exercises.push(exercise);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub enum ExerciseType {
    Bodyweight,
    Weighted,
    Assisted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct Exercise {
    id: Option<i64>,
    name: String,
    sets: Vec<Set>,
    exercise_type: ExerciseType,
}

impl Exercise {
    pub fn bodyweight(name: impl Into<String>) -> Exercise {
        Exercise {
            id: None,
            name: name.into(),
            sets: vec![],
            exercise_type: ExerciseType::Bodyweight,
        }
    }

    pub fn weighted(name: impl Into<String>) -> Exercise {
        Exercise {
            id: None,
            name: name.into(),
            sets: vec![],
            exercise_type: ExerciseType::Weighted,
        }
    }

    pub fn assisted(name: impl Into<String>) -> Exercise {
        Exercise {
            id: None,
            name: name.into(),
            sets: vec![],
            exercise_type: ExerciseType::Assisted,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sets(&self) -> &Vec<Set> {
        &self.sets
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
#[frb]
pub enum Set {
    Bodyweight { reps: i32 },
    Weighted { reps: i32, weight: i64 },
    Assisted { reps: i32, assistance: i64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_workout_works() {
        let workout = Workout::new("test workout");

        assert_eq!(workout.name, "test workout");
        assert_eq!(workout.exercises.len(), 0);
    }

    #[test]
    fn add_exercise_to_workout() {
        let mut workout = Workout::new("test workout");

        workout.add_exercise(Exercise::bodyweight("Pull-Up"));

        assert_eq!(workout.exercises.len(), 1);
        assert_eq!(workout.exercises[0].name(), "Pull-Up");
    }

    #[test]
    fn create_microcycle_works() {
        let microcycle = Microcycle::new("test microcycle");

        assert_eq!(microcycle.name, "test microcycle");
        assert_eq!(microcycle.workouts.len(), 0);
    }

    #[test]
    fn add_workout_to_microcycle() {
        let mut microcycle = Microcycle::new("test microcycle");

        let workout = Workout::new("Workout 1");
        microcycle.add_workout(workout);

        assert_eq!(microcycle.workouts.len(), 1);
        assert_eq!(microcycle.workouts[0].name, "Workout 1");
    }

    #[test]
    fn create_mesocycle_works() {
        let mesocycle = Mesocycle::new("test mesocycle");

        assert_eq!(mesocycle.name, "test mesocycle");
        assert_eq!(mesocycle.microcycles.len(), 0);
    }

    #[test]
    fn add_microcycle_to_mesocycle() {
        let mut mesocycle = Mesocycle::new("test mesocycle");

        let microcycle = Microcycle::new("Microcycle 1");
        mesocycle.add_microcycle(microcycle);

        assert_eq!(mesocycle.microcycles.len(), 1);
        assert_eq!(mesocycle.microcycles[0].name, "Microcycle 1");
    }

    #[test]
    fn add_set_to_exercise_with_matching_types_works() {
        let mut exercise = Exercise::bodyweight("Squat");
        assert_eq!(exercise.sets().len(), 0);

        exercise.add_set(Set::Bodyweight { reps: 42 }).unwrap();

        assert_eq!(exercise.sets().len(), 1);
        assert_eq!(exercise.sets()[0], Set::Bodyweight { reps: 42 });
    }

    #[test]
    fn add_set_to_exercise_with_mismatching_types_causes_error() {
        let mut exercise = Exercise::weighted("Squat");

        let result = exercise.add_set(Set::Bodyweight { reps: 42 });

        assert!(
            result.is_err(),
            "Should return error when set type doesn't match exercise type"
        )
    }
}
