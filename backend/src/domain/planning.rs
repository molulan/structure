use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mesocycle {
    id: i64,
    name: String,
    microcycles: Vec<Microcycle>,
}

impl Mesocycle {    
    pub fn id(&self) -> i64 {
        self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn microcycles(&self) -> &[Microcycle] {
        &self.microcycles
    }
    
    pub fn new(name: impl Into<String>, id: i64) -> Mesocycle {
        Mesocycle {
            id: id,
            name: name.into(),
            microcycles: Vec::new(),
        }
    }

    pub fn add_microcycle(&mut self, microcycle: Microcycle) {
        self.microcycles.push(microcycle);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Microcycle {
    id: Option<i64>,
    name: String,
    workouts: Vec<Workout>,
}

impl Microcycle {
    pub fn id(&self) -> Option<i64> {
        self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn workouts(&self) -> &[Workout] {
        &self.workouts
    }
    
    pub fn new(name: impl Into<String>) -> Microcycle {
        Microcycle {
            id: None,
            name: name.into(),
            workouts: Vec::new(),
        }
    }

    pub fn add_workout(&mut self, workout: Workout) {
        self.workouts.push(workout);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workout {
    id: Option<i64>,
    name: String,
    exercises: Vec<Exercise>,
}

impl Workout {
    pub fn id(&self) -> Option<i64> {
        self.id
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn exercises(&self) -> &[Exercise] {
        &self.exercises
    }
    
    pub fn new(name: impl Into<String>) -> Workout {
        Workout {
            id: None,
            name: name.into(),
            exercises: Vec::new(),
        }
    }

    pub fn add_exercise(&mut self, exercise: Exercise) {
        self.exercises.push(exercise);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ExerciseType {
    Bodyweight,
    Weighted,
    Assisted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Exercise {
    id: Option<i64>,
    name: String,
    sets: Vec<Set>,
    exercise_type: ExerciseType,
}

impl Exercise {
    pub fn id(&self) -> Option<i64> {
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
    
    pub fn bodyweight(name: impl Into<String>) -> Exercise {
        Exercise {
            id: None,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Bodyweight,
        }
    }

    pub fn weighted(name: impl Into<String>) -> Exercise {
        Exercise {
            id: None,
            name: name.into(),
            sets: Vec::new(),
            exercise_type: ExerciseType::Weighted,
        }
    }

    pub fn assisted(name: impl Into<String>) -> Exercise {
        Exercise {
            id: None,
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
    Lbs
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
        let mesocycle = Mesocycle::new("test mesocycle", 1);

        assert_eq!(mesocycle.name(), "test mesocycle");
        assert_eq!(mesocycle.microcycles.len(), 0);
        assert_eq!(mesocycle.id(), 1)
    }

    #[test]
    fn add_microcycle_to_mesocycle() {
        let mut mesocycle = Mesocycle::new("test mesocycle", 1);

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
