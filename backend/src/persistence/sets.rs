use crate::{
    domain::planning::{Effort, Load, Rir, Rpe, Set, SetType, Weight, WeightUnit},
    errors::SetError,
};
use rusqlite::{Connection, params};

pub(super) fn create_planned_sets_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS planned_sets (
            id INTEGER PRIMARY KEY,
            planned_exercise_id INTEGER NOT NULL REFERENCES planned_exercises(id),
            position INTEGER NOT NULL,
            set_type TEXT NOT NULL CHECK(
                set_type IN ('Regular', 'Myorep', 'MyorepMatch', 'Drop')
            ),
            load_type TEXT NOT NULL CHECK(
                load_type IN ('Bodyweight', 'WeightedBodyweight', 'AssistedBodyweight', 'Weighted')
            ),
            weight_value REAL,
            weight_unit TEXT CHECK(weight_unit IN ('Kg', 'Lbs')),
            reps INTEGER,
            effort_type TEXT CHECK(effort_type IN ('Rir', 'Rpe')),
            effort_value INTEGER,
            UNIQUE(planned_exercise_id, position),
            CHECK((weight_value IS NULL) = (weight_unit IS NULL))
        )",
        [],
    )?;
    Ok(())
}

fn set_type_to_str(set_type: SetType) -> &'static str {
    match set_type {
        SetType::Regular { .. } => "Regular",
        SetType::Myorep => "Myorep",
        SetType::MyorepMatch => "MyorepMatch",
        SetType::Drop { .. } => "Drop",
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

fn planned_exercise_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM planned_exercises WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create_planned_set(
    conn: &Connection,
    planned_exercise_id: i64,
    load: Load,
    reps: Option<u32>,
    set_type: SetType,
) -> Result<Set, SetError> {
    if !planned_exercise_exists(conn, planned_exercise_id)? {
        return Err(SetError::AssociatedPlannedExerciseNotFound {
            id: planned_exercise_id,
        });
    }

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM planned_sets WHERE planned_exercise_id = ?1",
        [planned_exercise_id],
        |row| row.get(0),
    )?;
    let position = u32::try_from(count)
        .expect("COUNT(*) is always non-negative and no exercise will have 4 billion sets");

    let set_type_str = set_type_to_str(set_type);

    let effort = match set_type {
        SetType::Regular { effort } => effort,
        SetType::Myorep => None,
        SetType::MyorepMatch => None,
        SetType::Drop { effort } => effort,
    };

    let (effort_type_str, effort_value): (Option<&'static str>, Option<i64>) = match effort {
        None => (None, None),
        Some(e) => {
            let t = effort_type_to_str(&e);
            let v: i64 = match e {
                Effort::Rpe(rpe) => rpe.value() as i64,
                Effort::Rir(rir) => rir.value() as i64,
            };
            (Some(t), Some(v))
        }
    };

    let load_type_str = load_type_to_str(&load);
    let (weight_value, weight_unit_str): (Option<f64>, Option<&'static str>) = match load {
        Load::Bodyweight => (None, None),
        Load::WeightedBodyweight { added_weight: w }
        | Load::AssistedBodyweight { assistance: w }
        | Load::Weighted { weight: w } => w.map_or((None, None), |w| {
            (Some(w.value()), Some(weight_unit_to_str(w.unit())))
        }),
    };

    let reps_db: Option<i64> = reps.map(|r| r as i64);

    conn.execute(
        "INSERT INTO planned_sets
            (planned_exercise_id, position, set_type, load_type,
             weight_value, weight_unit, reps, effort_type, effort_value)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            planned_exercise_id,
            position,
            set_type_str,
            load_type_str,
            weight_value,
            weight_unit_str,
            reps_db,
            effort_type_str,
            effort_value,
        ],
    )?;

    let id = conn.last_insert_rowid();
    Ok(Set::new(id, position, load, reps, set_type))
}

fn row_to_set(row: &rusqlite::Row<'_>) -> rusqlite::Result<Set> {
    let id = row.get(0)?;
    let position: i64 = row.get(1)?;
    let position = u32::try_from(position)
        .expect("COUNT(*) is always non-negative and no exercise will have 4 billion sets");
    let set_type: String = row.get(2)?;
    let load_type: String = row.get(3)?;
    let weight_value: Option<f64> = row.get(4)?;
    let weight_unit: Option<String> = row.get(5)?;
    let reps: Option<i64> = row.get(6)?;
    let effort_type: Option<String> = row.get(7)?;
    let effort_value: Option<i64> = row.get(8)?;

    let weight = weight_value
        .zip(weight_unit)
        .map(|(value, unit)| Weight::new(value, weight_unit_from_str(&unit)));

    let load = match load_type.as_str() {
        "Bodyweight" => Load::Bodyweight,
        "WeightedBodyweight" => Load::WeightedBodyweight {
            added_weight: weight,
        },
        "AssistedBodyweight" => Load::AssistedBodyweight { assistance: weight },
        "Weighted" => Load::Weighted { weight },
        other => panic!("unknown load_type in DB: {other}"),
    };

    let effort = match effort_type {
        None => None,
        Some(s) => match s.as_str() {
            "Rir" => {
                let v = effort_value.expect("effort_value is NULL but effort_type is 'Rir'");
                let v = i8::try_from(v).expect("effort_value out of i8 range");
                Some(Effort::Rir(Rir::new(v).expect("invalid Rir value in DB")))
            }
            "Rpe" => {
                let v = effort_value.expect("effort_value is NULL but effort_type is 'Rpe'");
                let v = u8::try_from(v).expect("effort_value out of u8 range");
                Some(Effort::Rpe(Rpe::new(v).expect("invalid Rpe value in DB")))
            }
            other => panic!("unknown effort_type in DB: {other}"),
        },
    };

    let reps: Option<u32> = reps.map(|r| u32::try_from(r).expect("reps out of u32 range"));

    let set_type = match set_type.as_str() {
        "Regular" => SetType::Regular { effort },
        "Myorep" => SetType::Myorep,
        "MyorepMatch" => SetType::MyorepMatch,
        "Drop" => SetType::Drop { effort },
        other => panic!("unknown set_type in DB: {other}"),
    };

    Ok(Set::new(id, position, load, reps, set_type))
}

pub fn list_planned_sets(
    conn: &Connection,
    planned_exercise_id: i64,
) -> Result<Vec<Set>, SetError> {
    if !planned_exercise_exists(conn, planned_exercise_id)? {
        return Err(SetError::AssociatedPlannedExerciseNotFound {
            id: planned_exercise_id,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT id, position, set_type, load_type, weight_value, weight_unit, reps, effort_type, effort_value
         FROM planned_sets
         WHERE planned_exercise_id = ?1
         ORDER BY position ASC",
    )?;

    let rows = stmt.query_map([planned_exercise_id], |row| row_to_set(row))?;

    let mut sets = Vec::new();
    for row in rows {
        sets.push(row?);
    }
    Ok(sets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::{ExerciseType, MesocycleMode, PlannedExercise, Weight, WeightUnit},
        persistence::{
            exercises::{create_exercise, create_planned_exercise},
            mesocycles::create_mesocycle,
            microcycles::create_microcycle,
            sqlite,
            workouts::create_workout,
        },
    };

    fn setup_test_db() -> Connection {
        sqlite::init_db(":memory:").expect("failed to create test database")
    }

    fn create_test_planned_exercise(conn: &Connection) -> PlannedExercise {
        let mesocycle = create_mesocycle(conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");
        let microcycle =
            create_microcycle(conn, mesocycle.id()).expect("microcycle creation should succeed");
        let workout = create_workout(conn, microcycle.id(), "Test Workout")
            .expect("workout creation should succeed");
        let exercise = create_exercise(conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let planned = create_planned_exercise(conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");
        planned
    }

    #[test]
    fn create_planned_set_on_existing_planned_exercise_succeeds() {
        let conn = setup_test_db();
        let planned_exercise = create_test_planned_exercise(&conn);

        let result = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted {
                weight: Some(Weight::new(100.0, WeightUnit::Kg)),
            },
            Some(5),
            SetType::Regular { effort: None },
        );

        assert!(result.is_ok());
    }

    #[test]
    fn create_planned_set_for_nonexistent_planned_exercise_returns_error() {
        let conn = setup_test_db();

        let result = create_planned_set(
            &conn,
            9999,
            Load::Bodyweight,
            Some(10),
            SetType::Regular { effort: None },
        );

        assert!(matches!(
            result,
            Err(SetError::AssociatedPlannedExerciseNotFound { .. })
        ));
    }

    #[test]
    fn first_planned_set_gets_position_0() {
        let conn = setup_test_db();
        let planned_exercise = create_test_planned_exercise(&conn);

        let set = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted { weight: None },
            Some(8),
            SetType::Regular { effort: None },
        )
        .expect("set creation should succeed");

        assert_eq!(set.position(), 0);
    }

    #[test]
    fn multiple_planned_sets_for_same_exercise_get_sequential_positions() {
        let conn = setup_test_db();
        let planned_exercise = create_test_planned_exercise(&conn);

        let set_1 = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted { weight: None },
            Some(5),
            SetType::Regular { effort: None },
        )
        .expect("first set creation should succeed");
        let set_2 = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted { weight: None },
            Some(5),
            SetType::Regular { effort: None },
        )
        .expect("second set creation should succeed");
        let set_3 = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted { weight: None },
            Some(5),
            SetType::Myorep,
        )
        .expect("third set creation should succeed");

        assert_eq!(set_1.position(), 0);
        assert_eq!(set_2.position(), 1);
        assert_eq!(set_3.position(), 2);
    }

    #[test]
    fn list_planned_sets_returns_empty_list_for_planned_exercise_with_no_sets() {
        let conn = setup_test_db();
        let planned_exercise = create_test_planned_exercise(&conn);

        let result = list_planned_sets(&conn, planned_exercise.id())
            .expect("listing sets for an existing planned exercise should succeed");

        assert!(result.is_empty());
    }

    #[test]
    fn list_planned_sets_returns_error_when_planned_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = list_planned_sets(&conn, 9999);

        assert!(matches!(
            result,
            Err(SetError::AssociatedPlannedExerciseNotFound { .. })
        ));
    }

    #[test]
    fn list_planned_sets_returns_all_sets_for_a_planned_exercise() {
        let conn = setup_test_db();
        let planned_exercise = create_test_planned_exercise(&conn);

        let set_1 = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted {
                weight: Some(Weight::new(80.0, WeightUnit::Kg)),
            },
            Some(5),
            SetType::Regular { effort: None },
        )
        .expect("first set creation should succeed");
        let set_2 = create_planned_set(
            &conn,
            planned_exercise.id(),
            Load::Weighted { weight: None },
            Some(8),
            SetType::Drop { effort: None },
        )
        .expect("second set creation should succeed");

        let result =
            list_planned_sets(&conn, planned_exercise.id()).expect("listing sets should succeed");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], set_1);
        assert_eq!(result[1], set_2);
    }

    #[test]
    fn list_planned_sets_returns_only_sets_belonging_to_the_requested_planned_exercise() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");
        let microcycle =
            create_microcycle(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let workout = create_workout(&conn, microcycle.id(), "Test Workout")
            .expect("workout creation should succeed");
        let exercise_1 = create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let target_planned = create_planned_exercise(&conn, workout.id(), exercise_1.id())
            .expect("first planned exercise creation should succeed");
        let other_planned = create_planned_exercise(&conn, workout.id(), exercise_2.id())
            .expect("second planned exercise creation should succeed");

        let target_set = create_planned_set(
            &conn,
            target_planned.id(),
            Load::Weighted { weight: None },
            Some(5),
            SetType::Regular { effort: None },
        )
        .expect("set creation for target exercise should succeed");
        let _ = create_planned_set(
            &conn,
            other_planned.id(),
            Load::Weighted { weight: None },
            Some(10),
            SetType::Myorep,
        )
        .expect("set creation for other exercise should succeed");

        let result =
            list_planned_sets(&conn, target_planned.id()).expect("listing sets should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], target_set);
    }
}
