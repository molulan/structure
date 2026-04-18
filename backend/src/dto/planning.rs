use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

use crate::domain::planning::{Mesocycle, Microcycle, Workout, ExerciseType, Exercise, Set, Weight, WeightUnit};


#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MesocycleDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) microcycles: Vec<MicrocycleDTO>,
}

impl From<&Mesocycle> for MesocycleDTO {
    fn from(value: &Mesocycle) -> Self {
        MesocycleDTO { 
            id: value.id(),
            name: value.name().to_owned(),
            microcycles: value
                .microcycles()
                .iter()
                .map(MicrocycleDTO::from)
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MicrocycleDTO {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) workouts: Vec<WorkoutDTO>,
}

impl From<&Microcycle> for MicrocycleDTO {
    fn from(value: &Microcycle) -> Self {
        MicrocycleDTO { 
            id: value.id(),
            name: value.name().to_owned(),
            workouts: value
                .workouts()
                .iter()
                .map(WorkoutDTO::from)
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct WorkoutDTO {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) exercises: Vec<ExerciseDTO>,
}

impl From<&Workout> for WorkoutDTO {
    fn from(value: &Workout) -> Self {
        WorkoutDTO { 
            id: value.id(),
            name: value.name().to_owned(),
            exercises: value
                .exercises()
                .iter()
                .map(ExerciseDTO::from)
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub enum ExerciseTypeDTO {
    Bodyweight,
    Weighted,
    Assisted,
}

impl From<&ExerciseType> for ExerciseTypeDTO {
    fn from(value: &ExerciseType) -> Self {
        match value {
            ExerciseType::Assisted => ExerciseTypeDTO::Assisted,
            ExerciseType::Bodyweight => ExerciseTypeDTO::Bodyweight,
            ExerciseType::Weighted => ExerciseTypeDTO::Weighted,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct ExerciseDTO {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) exercise_type: ExerciseTypeDTO,
    pub(crate) sets: Vec<SetDTO>,
}

impl From<&Exercise> for ExerciseDTO {
    fn from(value: &Exercise) -> Self {
        ExerciseDTO { 
            id: value.id(),
            name: value.name().to_owned(),
            exercise_type: ExerciseTypeDTO::from(&value.exercise_type()),
            sets: value
                .sets()
                .iter()
                .map(SetDTO::from)
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[frb]
pub enum SetDTO {
    Bodyweight { reps: u32 },
    Weighted { reps: u32, weight: WeightDTO },
    Assisted { reps: u32, assistance: WeightDTO },
}

impl From<&Set> for SetDTO {
    fn from(value: &Set) -> Self {
        match value {
            Set::Assisted { reps, assistance } => {
                SetDTO::Assisted { 
                    reps: *reps, 
                    assistance: WeightDTO::from(assistance) }
            }
            Set::Bodyweight { reps } => {
                SetDTO::Bodyweight { reps: *reps }
            }
            Set::Weighted { reps, weight } => {
                SetDTO::Weighted { 
                    reps: *reps,
                    weight: WeightDTO::from(weight) }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub struct WeightDTO {
    pub(crate) value: f64,
    pub(crate) unit: WeightUnitDTO,
}

impl From<&Weight> for WeightDTO {
    fn from(value: &Weight) -> Self {
        WeightDTO { 
            value: value.value(),
            unit: WeightUnitDTO::from(&value.unit()) }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum WeightUnitDTO {
    Kg,
    Lbs
}

impl From<&WeightUnit> for WeightUnitDTO {
    fn from(value: &WeightUnit) -> Self {
        match value {
            WeightUnit::Kg => WeightUnitDTO::Kg,
            WeightUnit::Lbs => WeightUnitDTO::Lbs,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
