use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};

use crate::domain::planning::{
    Effort, Exercise, ExerciseType, Load, Mesocycle, Microcycle, Rir, Rpe, Set, Weight, WeightUnit,
    Workout,
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
    pub(crate) position: u32,
}

impl From<&Workout> for WorkoutDTO {
    fn from(value: &Workout) -> Self {
        WorkoutDTO {
            id: value.id(),
            name: value.name().to_owned(),
            position: value.position(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum ExerciseTypeDTO {
    Bodyweight,
    WeightedBodyweight,
    AssistedBodyweight,
    Weighted,
}

impl From<ExerciseType> for ExerciseTypeDTO {
    fn from(value: ExerciseType) -> Self {
        match value {
            ExerciseType::Bodyweight => ExerciseTypeDTO::Bodyweight,
            ExerciseType::WeightedBodyweight => ExerciseTypeDTO::WeightedBodyweight,
            ExerciseType::AssistedBodyweight => ExerciseTypeDTO::AssistedBodyweight,
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
    pub(crate) position: u32,
    pub(crate) sets: Vec<SetDTO>,
}

impl From<&Exercise> for ExerciseDTO {
    fn from(value: &Exercise) -> Self {
        ExerciseDTO {
            id: value.id(),
            name: value.name().to_owned(),
            exercise_type: ExerciseTypeDTO::from(value.exercise_type()),
            position: value.position(),
            sets: value.sets().iter().copied().map(SetDTO::from).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum SetDTO {
    Regular {
        load: LoadDTO,
        reps: Option<u32>,
        effort: Option<EffortDTO>,
    },
    Myorep {
        load: LoadDTO,
        reps: Option<u32>,
    },
    MyorepMatch {
        load: LoadDTO,
        reps: Option<u32>,
    },
    Drop {
        load: LoadDTO,
        reps: Option<u32>,
        effort: Option<EffortDTO>,
    },
}

impl From<Set> for SetDTO {
    fn from(value: Set) -> Self {
        match value {
            Set::Regular { load, reps, effort } => SetDTO::Regular {
                load: LoadDTO::from(load),
                reps,
                effort: effort.map(EffortDTO::from),
            },
            Set::Drop { load, reps, effort } => SetDTO::Drop {
                load: LoadDTO::from(load),
                reps,
                effort: effort.map(EffortDTO::from),
            },
            Set::MyorepMatch { load, reps } => SetDTO::MyorepMatch {
                load: LoadDTO::from(load),
                reps,
            },
            Set::Myorep { load, reps } => SetDTO::Myorep {
                load: LoadDTO::from(load),
                reps,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum LoadDTO {
    Bodyweight,
    WeightedBodyweight { added_weight: Option<WeightDTO> },
    AssistedBodyweight { assistance: Option<WeightDTO> },
    Weighted { weight: Option<WeightDTO> },
}

impl From<Load> for LoadDTO {
    fn from(value: Load) -> Self {
        match value {
            Load::Bodyweight => LoadDTO::Bodyweight,
            Load::WeightedBodyweight { added_weight } => LoadDTO::WeightedBodyweight {
                added_weight: added_weight.map(WeightDTO::from),
            },
            Load::AssistedBodyweight { assistance } => LoadDTO::AssistedBodyweight {
                assistance: assistance.map(WeightDTO::from),
            },
            Load::Weighted { weight } => LoadDTO::Weighted {
                weight: weight.map(WeightDTO::from),
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

impl From<Weight> for WeightDTO {
    fn from(value: Weight) -> Self {
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum EffortDTO {
    Rir(RirDTO),
    Rpe(RpeDTO),
}

impl From<Effort> for EffortDTO {
    fn from(value: Effort) -> Self {
        match value {
            Effort::Rir(rir) => EffortDTO::Rir(RirDTO::from(rir)),
            Effort::Rpe(rpe) => EffortDTO::Rpe(RpeDTO::from(rpe)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub struct RpeDTO(u8);

impl From<Rpe> for RpeDTO {
    fn from(value: Rpe) -> Self {
        RpeDTO(value.value())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub struct RirDTO(i8);

impl From<Rir> for RirDTO {
    fn from(value: Rir) -> Self {
        RirDTO(value.value())
    }
}
