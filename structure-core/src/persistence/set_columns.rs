use crate::domain::planning::{Effort, Load, Rir, Rpe, SetType, Weight, WeightUnit};

/// The persisted column values for a set's `load` and `set_type`.
///
/// `planned_sets` and `logged_sets` store identically shaped set data; this
/// module is the single `Load`/`SetType`/`Effort` ↔ column mapping shared by
/// both. Callers own the columns that differ between the two tables (reps,
/// `planned_set_id`).
pub(super) struct SetColumns {
    pub set_type: &'static str,
    pub load_type: &'static str,
    pub weight_value: Option<f64>,
    pub weight_unit: Option<&'static str>,
    pub effort_type: Option<&'static str>,
    pub effort_value: Option<i64>,
}

impl SetColumns {
    pub(super) fn from_set(load: Load, set_type: SetType) -> SetColumns {
        let effort = match set_type {
            SetType::Regular { effort } => effort,
            SetType::Myorep | SetType::MyorepMatch | SetType::Drop => None,
        };

        let (effort_type, effort_value) = match effort {
            None => (None, None),
            Some(effort) => {
                let value = match effort {
                    Effort::Rpe(rpe) => rpe.value() as i64,
                    Effort::Rir(rir) => rir.value() as i64,
                };
                (Some(effort_type_to_str(&effort)), Some(value))
            }
        };

        let (weight_value, weight_unit) = match load {
            Load::Bodyweight => (None, None),
            Load::WeightedBodyweight {
                added_weight: weight,
            }
            | Load::AssistedBodyweight { assistance: weight }
            | Load::Weighted { weight } => weight.map_or((None, None), |weight| {
                (
                    Some(weight.value()),
                    Some(weight_unit_to_str(weight.unit())),
                )
            }),
        };

        SetColumns {
            set_type: set_type_to_str(set_type),
            load_type: load_type_to_str(&load),
            weight_value,
            weight_unit,
            effort_type,
            effort_value,
        }
    }
}

pub(super) fn to_load_and_set_type(
    set_type: &str,
    load_type: &str,
    weight_value: Option<f64>,
    weight_unit: Option<String>,
    effort_type: Option<&str>,
    effort_value: Option<i64>,
) -> (Load, SetType) {
    let weight = weight_value
        .zip(weight_unit)
        .map(|(value, unit)| Weight::new(value, weight_unit_from_str(&unit)));

    let load = match load_type {
        "Bodyweight" => Load::Bodyweight,
        "WeightedBodyweight" => Load::WeightedBodyweight {
            added_weight: weight,
        },
        "AssistedBodyweight" => Load::AssistedBodyweight { assistance: weight },
        "Weighted" => Load::Weighted { weight },
        other => panic!("unknown load_type in DB: {other}"),
    };

    let effort = effort_from_columns(effort_type, effort_value);

    let set_type = match set_type {
        "Regular" => SetType::Regular { effort },
        "Myorep" => SetType::Myorep,
        "MyorepMatch" => SetType::MyorepMatch,
        "Drop" => SetType::Drop,
        other => panic!("unknown set_type in DB: {other}"),
    };

    (load, set_type)
}

fn set_type_to_str(set_type: SetType) -> &'static str {
    match set_type {
        SetType::Regular { .. } => "Regular",
        SetType::Myorep => "Myorep",
        SetType::MyorepMatch => "MyorepMatch",
        SetType::Drop => "Drop",
    }
}

fn load_type_to_str(load: &Load) -> &'static str {
    match load {
        Load::Bodyweight => "Bodyweight",
        Load::WeightedBodyweight { .. } => "WeightedBodyweight",
        Load::AssistedBodyweight { .. } => "AssistedBodyweight",
        Load::Weighted { .. } => "Weighted",
    }
}

fn weight_unit_to_str(unit: WeightUnit) -> &'static str {
    match unit {
        WeightUnit::Kg => "Kg",
        WeightUnit::Lbs => "Lbs",
    }
}

fn weight_unit_from_str(s: &str) -> WeightUnit {
    match s {
        "Kg" => WeightUnit::Kg,
        "Lbs" => WeightUnit::Lbs,
        other => panic!("unknown weight_unit in DB: {other}"),
    }
}

fn effort_type_to_str(effort: &Effort) -> &'static str {
    match effort {
        Effort::Rir(..) => "Rir",
        Effort::Rpe(..) => "Rpe",
    }
}

fn effort_from_columns(effort_type: Option<&str>, effort_value: Option<i64>) -> Option<Effort> {
    match effort_type {
        None => None,
        Some("Rir") => {
            let v = effort_value.expect("effort_value is NULL but effort_type is 'Rir'");
            let v = i8::try_from(v).expect("effort_value out of i8 range");
            Some(Effort::Rir(Rir::new(v).expect("invalid Rir value in DB")))
        }
        Some("Rpe") => {
            let v = effort_value.expect("effort_value is NULL but effort_type is 'Rpe'");
            let v = u8::try_from(v).expect("effort_value out of u8 range");
            Some(Effort::Rpe(Rpe::new(v).expect("invalid Rpe value in DB")))
        }
        Some(other) => panic!("unknown effort_type in DB: {other}"),
    }
}
