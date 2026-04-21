use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

use crate::domain::planning::{
    Exercise, ExerciseType, Mesocycle, Microcycle, Set, Weight, WeightUnit, Workout,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MesocycleDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
}

impl From<&Mesocycle> for MesocycleDTO {
    fn from(value: &Mesocycle) -> Self {
        MesocycleDTO {
            id: value.id(),
            name: value.name().to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MicrocycleDTO {
    pub(crate) id: i64,
    pub(crate) position: u32,
}

impl From<&Microcycle> for MicrocycleDTO {
    fn from(value: &Microcycle) -> Self {
        MicrocycleDTO {
            id: value.id(),
            position: value.position(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct WorkoutDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
}

impl From<&Workout> for WorkoutDTO {
    fn from(value: &Workout) -> Self {
        WorkoutDTO {
            id: value.id(),
            name: value.name().to_owned(),
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

impl From<ExerciseType> for ExerciseTypeDTO {
    fn from(value: ExerciseType) -> Self {
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
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) exercise_type: ExerciseTypeDTO,
    pub(crate) sets: Vec<SetDTO>,
}

impl From<&Exercise> for ExerciseDTO {
    fn from(value: &Exercise) -> Self {
        ExerciseDTO {
            id: value.id(),
            name: value.name().to_owned(),
            exercise_type: ExerciseTypeDTO::from(value.exercise_type()),
            sets: value.sets().iter().map(SetDTO::from).collect(),
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
            Set::Assisted { reps, assistance } => SetDTO::Assisted {
                reps: *reps,
                assistance: WeightDTO::from(assistance),
            },
            Set::Bodyweight { reps } => SetDTO::Bodyweight { reps: *reps },
            Set::Weighted { reps, weight } => SetDTO::Weighted {
                reps: *reps,
                weight: WeightDTO::from(weight),
            },
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
            unit: WeightUnitDTO::from(value.unit()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum WeightUnitDTO {
    Kg,
    Lbs,
}

impl From<WeightUnit> for WeightUnitDTO {
    fn from(value: WeightUnit) -> Self {
        match value {
            WeightUnit::Kg => WeightUnitDTO::Kg,
            WeightUnit::Lbs => WeightUnitDTO::Lbs,
        }
    }
}
