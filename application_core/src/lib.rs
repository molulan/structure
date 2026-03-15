#[derive(Debug)]
pub struct Mesocycle {
    pub name: String,
    pub microcycles: Vec<Microcycle>,
}

impl Mesocycle {
    pub fn new(name: &str) -> Mesocycle {
        Mesocycle {
            name: String::from(name),
            microcycles: vec![],
        }
    }

    pub fn add_microcycle(&mut self, microcycle: Microcycle) {
        self.microcycles.push(microcycle);
    }
}

#[derive(Debug)]
pub struct Microcycle {
    pub name: String,
    pub workouts: Vec<Workout>,
}

impl Microcycle {
    pub fn new(name: &str) -> Microcycle {
        Microcycle {
            name: String::from(name),
            workouts: vec![],
        }
    }

    pub fn add_workout(&mut self, workout: Workout) {
        self.workouts.push(workout);
    }
}

#[derive(Debug)]
pub struct Workout {
    pub name: String,
    pub exercises: Vec<Exercise>,
}

impl Workout {
    pub fn new(name: &str) -> Workout {
        Workout {
            name: String::from(name),
            exercises: vec![],
        }
    }

    pub fn add_exercise(&mut self, exercise: Exercise) {
        self.exercises.push(exercise);
    }
}

#[derive(Debug, PartialEq)]
pub enum Exercise {
    Bodyweight { name: String, sets: Vec<Set> },
    Assisted { name: String, sets: Vec<Set> },
    Weighted { name: String, sets: Vec<Set> },
}

impl Exercise {
    pub fn name(&self) -> &str {
        match self {
            Exercise::Bodyweight { name, .. } => name,
            Exercise::Weighted { name, .. } => name,
            Exercise::Assisted { name, .. } => name,
        }
    }

    pub fn sets(&self) -> &Vec<Set> {
        match self {
            Exercise::Bodyweight { sets, .. } => sets,
            Exercise::Weighted { sets, .. } => sets,
            Exercise::Assisted { sets, .. } => sets,
        }
    }

    pub fn bodyweight(name: &str) -> Exercise {
        Exercise::Bodyweight {
            name: String::from(name),
            sets: vec![],
        }
    }

    pub fn weighted(name: &str) -> Exercise {
        Exercise::Weighted {
            name: String::from(name),
            sets: vec![],
        }
    }

    pub fn assisted(name: &str) -> Exercise {
        Exercise::Assisted {
            name: String::from(name),
            sets: vec![],
        }
    }

    pub fn add_set(&mut self, set: Set) -> Result<(), String> {
        match (self, &set) {
            (Exercise::Bodyweight { sets, .. }, Set::Bodyweight { .. }) => {
                sets.push(set);
                Ok(())
            }
            (Exercise::Weighted { sets, .. }, Set::Weighted { .. }) => {
                sets.push(set);
                Ok(())
            }
            (Exercise::Assisted { sets, .. }, Set::Assisted { .. }) => {
                sets.push(set);
                Ok(())
            }
            (exercise, set) => Err(format!("Cannot add {:?}set to {:?}exercise", set, exercise)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Set {
    Bodyweight { reps: u32 },
    Weighted { reps: u32, weight: u32 },
    Assisted { reps: u32, assistence: u32 },
}

pub fn hello() -> String {
    String::from("Hello from the application Core!")
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
