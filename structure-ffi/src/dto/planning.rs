use flutter_rust_bridge::frb;
use serde::{Deserialize, Serialize};
use structure_core::domain::planning::{
    Effort, Exercise, ExerciseType, Load, MesocycleMode, Microcycle, PlannedExercise, Rir, Rpe,
    Set, SetType, Weight, WeightUnit, Workout,
};


#[derive(Serialize, Deserialize, Debug, Clone)]
#[frb]
pub struct MesocycleDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) mode: MesocycleModeDTO,
    pub(crate) microcycle_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum MesocycleModeDTO {
    Algorithmic,
    Manual,
}

impl From<MesocycleMode> for MesocycleModeDTO {
    fn from(value: MesocycleMode) -> Self {
        match value {
            MesocycleMode::Algorithmic => Self::Algorithmic,
            MesocycleMode::Manual => Self::Manual,
        }
    }
}

impl From<MesocycleModeDTO> for MesocycleMode {
    fn from(value: MesocycleModeDTO) -> Self {
        match value {
            MesocycleModeDTO::Algorithmic => Self::Algorithmic,
            MesocycleModeDTO::Manual => Self::Manual,
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

impl From<ExerciseTypeDTO> for ExerciseType {
    fn from(dto: ExerciseTypeDTO) -> Self {
        match dto {
            ExerciseTypeDTO::Bodyweight => Self::Bodyweight,
            ExerciseTypeDTO::WeightedBodyweight => Self::WeightedBodyweight,
            ExerciseTypeDTO::AssistedBodyweight => Self::AssistedBodyweight,
            ExerciseTypeDTO::Weighted => Self::Weighted,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[frb]
pub struct PlannedExerciseDTO {
    pub(crate) id: i64,
    pub(crate) exercise: ExerciseDTO,
    pub(crate) position: u32,
    pub(crate) sets: Vec<SetDTO>,
}

impl From<&PlannedExercise> for PlannedExerciseDTO {
    fn from(value: &PlannedExercise) -> Self {
        PlannedExerciseDTO {
            id: value.id(),
            exercise: ExerciseDTO::from(value.exercise()),
            position: value.position(),
            sets: value.sets().iter().copied().map(SetDTO::from).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[frb]
pub struct ExerciseDTO {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) exercise_type: ExerciseTypeDTO,
}

impl From<&Exercise> for ExerciseDTO {
    fn from(value: &Exercise) -> Self {
        ExerciseDTO {
            id: value.id(),
            name: value.name().to_owned(),
            exercise_type: ExerciseTypeDTO::from(value.exercise_type()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub struct SetDTO {
    pub(crate) id: i64,
    pub(crate) position: u32,
    pub(crate) load: LoadDTO,
    pub(crate) reps: Option<u32>,
    pub(crate) set_type: SetTypeDTO,
}

impl From<Set> for SetDTO {
    fn from(value: Set) -> Self {
        SetDTO {
            id: value.id(),
            position: value.position(),
            load: LoadDTO::from(value.load()),
            reps: value.reps(),
            set_type: SetTypeDTO::from(value.set_type()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub enum SetTypeDTO {
    Regular { effort: Option<EffortDTO> },
    Myorep,
    MyorepMatch,
    Drop { effort: Option<EffortDTO> },
}

impl From<SetType> for SetTypeDTO {
    fn from(value: SetType) -> Self {
        match value {
            SetType::Regular { effort } => Self::Regular {
                effort: effort.map(EffortDTO::from),
            },
            SetType::Myorep => Self::Myorep,
            SetType::MyorepMatch => Self::MyorepMatch,
            SetType::Drop { effort } => Self::Drop {
                effort: effort.map(EffortDTO::from),
            },
        }
    }
}

impl From<SetTypeDTO> for SetType {
    fn from(value: SetTypeDTO) -> Self {
        match value {
            SetTypeDTO::Regular { effort } => SetType::Regular {
                effort: effort.map(Effort::from),
            },
            SetTypeDTO::Myorep => SetType::Myorep,
            SetTypeDTO::MyorepMatch => SetType::MyorepMatch,
            SetTypeDTO::Drop { effort } => SetType::Drop {
                effort: effort.map(Effort::from),
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

impl From<LoadDTO> for Load {
    fn from(value: LoadDTO) -> Self {
        match value {
            LoadDTO::Bodyweight => Load::Bodyweight,
            LoadDTO::WeightedBodyweight { added_weight } => Load::WeightedBodyweight {
                added_weight: added_weight.map(Weight::from),
            },
            LoadDTO::AssistedBodyweight { assistance } => Load::AssistedBodyweight {
                assistance: assistance.map(Weight::from),
            },
            LoadDTO::Weighted { weight } => Load::Weighted {
                weight: weight.map(Weight::from),
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

impl From<WeightDTO> for Weight {
    fn from(value: WeightDTO) -> Self {
        Weight::new(value.value, WeightUnit::from(value.unit))
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

impl From<WeightUnitDTO> for WeightUnit {
    fn from(value: WeightUnitDTO) -> Self {
        match value {
            WeightUnitDTO::Kg => Self::Kg,
            WeightUnitDTO::Lbs => Self::Lbs,
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

impl From<EffortDTO> for Effort {
    fn from(value: EffortDTO) -> Self {
        match value {
            EffortDTO::Rir(rir) => Effort::Rir(Rir::from(rir)),
            EffortDTO::Rpe(rpe) => Effort::Rpe(Rpe::from(rpe)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub struct RpeDTO(pub(crate) u8);

impl From<Rpe> for RpeDTO {
    fn from(value: Rpe) -> Self {
        RpeDTO(value.value())
    }
}

impl From<RpeDTO> for Rpe {
    fn from(value: RpeDTO) -> Self {
        Rpe::new(value.0).expect("RpeDTO value out of valid range (1..=11)")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[frb]
pub struct RirDTO(pub(crate) i8);

impl From<Rir> for RirDTO {
    fn from(value: Rir) -> Self {
        RirDTO(value.value())
    }
}

impl From<RirDTO> for Rir {
    fn from(value: RirDTO) -> Self {
        Rir::new(value.0).expect("RirDTO value out of valid range (-1..=10)")
    }
}
