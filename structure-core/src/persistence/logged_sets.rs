use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::{Effort, ExerciseType, Load, Rir, Rpe, SetType, Weight, WeightUnit};
use crate::domain::tracking::LoggedSet;
use crate::persistence::library_exercises::exercise_type_from_str;

#[derive(Debug, thiserror::Error)]
pub enum LoggedSetError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated logged exercise {id} not found")]
    AssociatedLoggedExerciseNotFound { id: i64 },
    #[error("associated planned set {id} not found")]
    AssociatedPlannedSetNotFound { id: i64 },
    #[error("logging drop sets is not yet supported")]
    DropLoggingNotSupported,
    #[error("logged set {id} not found")]
    NotFound { id: i64 },
    #[error(transparent)]
    Invalid(#[from] crate::domain::planning::SetValidationError),
}

pub(super) fn create_logged_sets_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logged_sets (
            id INTEGER PRIMARY KEY,
            logged_exercise_id INTEGER NOT NULL REFERENCES logged_exercises(id) ON DELETE CASCADE,
            planned_set_id INTEGER REFERENCES planned_sets(id) ON DELETE SET NULL,
            position INTEGER NOT NULL,
            set_type TEXT NOT NULL CHECK(
                set_type IN ('Regular', 'Myorep', 'MyorepMatch', 'Drop')
            ),
            load_type TEXT NOT NULL CHECK(
                load_type IN ('Bodyweight', 'WeightedBodyweight', 'AssistedBodyweight', 'Weighted')
            ),
            weight_value REAL,
            weight_unit TEXT CHECK(weight_unit IN ('Kg', 'Lbs')),
            reps INTEGER NOT NULL,
            effort_type TEXT CHECK(effort_type IN ('Rir', 'Rpe')),
            effort_value INTEGER,
            UNIQUE(logged_exercise_id, position),
            CHECK((weight_value IS NULL) = (weight_unit IS NULL))
        )",
        [],
    )?;
    Ok(())
}

fn logged_exercise_type(
    conn: &Connection,
    logged_exercise_id: i64,
) -> rusqlite::Result<Option<ExerciseType>> {
    conn.query_row(
        "SELECT le.exercise_type FROM logged_exercises lex
         JOIN library_exercises le ON le.id = lex.library_exercise_id
         WHERE lex.id = ?1",
        [logged_exercise_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|name| name.map(|name| exercise_type_from_str(&name)))
}

fn planned_set_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM planned_sets WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create(
    conn: &mut Connection,
    logged_exercise_id: i64,
    load: Load,
    reps: u32,
    set_type: SetType,
    planned_set_id: Option<i64>,
) -> Result<LoggedSet, LoggedSetError> {
    if matches!(set_type, SetType::Drop) {
        return Err(LoggedSetError::DropLoggingNotSupported);
    }

    let tx = conn.transaction()?;

    let Some(exercise_type) = logged_exercise_type(&tx, logged_exercise_id)? else {
        return Err(LoggedSetError::AssociatedLoggedExerciseNotFound {
            id: logged_exercise_id,
        });
    };

    if let Some(planned_id) = planned_set_id
        && !planned_set_exists(&tx, planned_id)?
    {
        return Err(LoggedSetError::AssociatedPlannedSetNotFound { id: planned_id });
    }

    let next_position: i64 = tx.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM logged_sets WHERE logged_exercise_id = ?1",
        [logged_exercise_id],
        |row| row.get(0),
    )?;
    let position = u32::try_from(next_position)
        .expect("positions are non-negative and no exercise will have 4 billion sets");

    let columns = set_columns(load, set_type);

    tx.execute(
        "INSERT INTO logged_sets
            (logged_exercise_id, planned_set_id, position, set_type, load_type,
             weight_value, weight_unit, reps, effort_type, effort_value)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            logged_exercise_id,
            planned_set_id,
            position,
            columns.set_type,
            columns.load_type,
            columns.weight_value,
            columns.weight_unit,
            reps,
            columns.effort_type,
            columns.effort_value,
        ],
    )?;

    let id = tx.last_insert_rowid();
    let set = LoggedSet::new(
        id,
        position,
        exercise_type,
        load,
        reps,
        set_type,
        planned_set_id,
    )?;
    tx.commit()?;
    Ok(set)
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<LoggedSet>, LoggedSetError> {
    let set = conn
        .query_row(
            "SELECT ls.id, ls.position, ls.set_type, ls.load_type, ls.weight_value,
                    ls.weight_unit, ls.reps, ls.effort_type, ls.effort_value, ls.planned_set_id,
                    le.exercise_type
             FROM logged_sets ls
             JOIN logged_exercises lex ON lex.id = ls.logged_exercise_id
             JOIN library_exercises le ON le.id = lex.library_exercise_id
             WHERE ls.id = ?1",
            [id],
            row_to_set,
        )
        .optional()?;

    Ok(set)
}

pub fn list(conn: &Connection, logged_exercise_id: i64) -> Result<Vec<LoggedSet>, LoggedSetError> {
    if logged_exercise_type(conn, logged_exercise_id)?.is_none() {
        return Err(LoggedSetError::AssociatedLoggedExerciseNotFound {
            id: logged_exercise_id,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT ls.id, ls.position, ls.set_type, ls.load_type, ls.weight_value,
                ls.weight_unit, ls.reps, ls.effort_type, ls.effort_value, ls.planned_set_id,
                le.exercise_type
         FROM logged_sets ls
         JOIN logged_exercises lex ON lex.id = ls.logged_exercise_id
         JOIN library_exercises le ON le.id = lex.library_exercise_id
         WHERE ls.logged_exercise_id = ?1
         ORDER BY ls.position ASC",
    )?;
    let rows = stmt.query_map([logged_exercise_id], row_to_set)?;

    let mut sets = Vec::new();
    for row in rows {
        sets.push(row?);
    }
    Ok(sets)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), LoggedSetError> {
    let deleted = conn.execute("DELETE FROM logged_sets WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(LoggedSetError::NotFound { id });
    }
    Ok(())
}

fn row_to_set(row: &rusqlite::Row<'_>) -> rusqlite::Result<LoggedSet> {
    let id = row.get(0)?;
    let position: i64 = row.get(1)?;
    let position = u32::try_from(position).expect("position stored in DB was originally a u32");
    let set_type: String = row.get(2)?;
    let load_type: String = row.get(3)?;
    let weight_value: Option<f64> = row.get(4)?;
    let weight_unit: Option<String> = row.get(5)?;
    let reps: i64 = row.get(6)?;
    let effort_type: Option<String> = row.get(7)?;
    let effort_value: Option<i64> = row.get(8)?;
    let planned_set_id: Option<i64> = row.get(9)?;
    let exercise_type: String = row.get(10)?;
    let exercise_type = exercise_type_from_str(&exercise_type);

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

    let effort = effort_from_columns(effort_type.as_deref(), effort_value);

    let set_type = match set_type.as_str() {
        "Regular" => SetType::Regular { effort },
        "Myorep" => SetType::Myorep,
        "MyorepMatch" => SetType::MyorepMatch,
        "Drop" => SetType::Drop,
        other => panic!("unknown set_type in DB: {other}"),
    };

    let reps = u32::try_from(reps).expect("reps out of u32 range");

    Ok(LoggedSet::new(
        id,
        position,
        exercise_type,
        load,
        reps,
        set_type,
        planned_set_id,
    )
    .expect("set load stored in DB was validated on write"))
}

/// The persisted column values for a logged set's `load` and `set_type`.
///
/// Duplicated from `persistence::sets`; once both planned and logged sets exist,
/// this `Load`/`SetType`/`Effort` ↔ column mapping should be extracted into a
/// shared helper used by both modules.
struct SetColumns {
    set_type: &'static str,
    load_type: &'static str,
    weight_value: Option<f64>,
    weight_unit: Option<&'static str>,
    effort_type: Option<&'static str>,
    effort_value: Option<i64>,
}

fn set_columns(load: Load, set_type: SetType) -> SetColumns {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::planning::{MesocycleMode, SetValidationError};
    use crate::persistence::{
        connection, library_exercises, logged_exercises, logged_sessions, mesocycles, microcycles,
        planned_exercises, sets, workouts,
    };

    const STARTED: &str = "2026-06-26T10:00:00Z";

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("failed to create test database")
    }

    /// A logged exercise referencing a `Weighted` library exercise.
    fn create_weighted_logged_exercise(conn: &Connection) -> i64 {
        let session = logged_sessions::create(conn, STARTED, None, None, None)
            .expect("session creation should succeed")
            .id();
        let exercise = library_exercises::create(conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        logged_exercises::create(conn, session, exercise.id(), None, None)
            .expect("logged exercise creation should succeed")
            .id()
    }

    fn weighted(value: f64) -> Load {
        Load::Weighted {
            weight: Some(Weight::new(value, WeightUnit::Kg)),
        }
    }

    fn create_planned_set(conn: &mut Connection) -> i64 {
        let mesocycle = mesocycles::create(conn, "Test Mesocycle", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let microcycle =
            microcycles::create(conn, mesocycle.id()).expect("microcycle creation should succeed");
        let workout = workouts::create(conn, microcycle.id(), "Push")
            .expect("workout creation should succeed");
        let exercise = library_exercises::create(conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let planned = planned_exercises::create(conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");
        sets::create(
            conn,
            planned.id(),
            weighted(100.0),
            Some(5),
            SetType::Regular { effort: None },
        )
        .expect("planned set creation should succeed")
        .id()
    }

    #[test]
    fn create_set_records_load_reps_and_position() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let set = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        )
        .expect("logged set creation should succeed");

        assert_eq!(set.position(), 0);
        assert_eq!(set.reps(), 5);
        assert_eq!(set.load(), weighted(100.0));
        assert_eq!(set.planned_set_id(), None);
    }

    #[test]
    fn create_set_with_effort_round_trips_through_the_database() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);
        let effort = Some(Effort::Rir(Rir::new(1).expect("valid rir")));

        let set = create(
            &mut conn,
            logged_exercise_id,
            weighted(102.5),
            4,
            SetType::Regular { effort },
            None,
        )
        .expect("logged set creation should succeed");

        let persisted = get(&conn, set.id())
            .expect("query should succeed")
            .expect("set should exist");
        assert_eq!(persisted.set_type(), SetType::Regular { effort });
        assert_eq!(persisted.load(), weighted(102.5));
        assert_eq!(persisted.reps(), 4);
    }

    #[test]
    fn create_myorep_set_succeeds() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let set = create(
            &mut conn,
            logged_exercise_id,
            weighted(60.0),
            18,
            SetType::Myorep,
            None,
        )
        .expect("logged set creation should succeed");

        assert_eq!(set.set_type(), SetType::Myorep);
    }

    #[test]
    fn create_drop_set_is_rejected() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let result = create(
            &mut conn,
            logged_exercise_id,
            weighted(80.0),
            8,
            SetType::Drop,
            None,
        );

        assert!(matches!(
            result,
            Err(LoggedSetError::DropLoggingNotSupported)
        ));
    }

    #[test]
    fn create_set_for_nonexistent_logged_exercise_returns_error() {
        let mut conn = setup_test_db();

        let result = create(
            &mut conn,
            9999,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        );

        assert!(matches!(
            result,
            Err(LoggedSetError::AssociatedLoggedExerciseNotFound { id: 9999 })
        ));
    }

    #[test]
    fn create_set_with_load_not_matching_exercise_type_returns_load_mismatch() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let result = create(
            &mut conn,
            logged_exercise_id,
            Load::Bodyweight,
            5,
            SetType::Regular { effort: None },
            None,
        );

        assert!(matches!(
            result,
            Err(LoggedSetError::Invalid(
                SetValidationError::LoadMismatch { .. }
            ))
        ));
    }

    #[test]
    fn create_set_linked_to_a_planned_set_stores_the_link() {
        let mut conn = setup_test_db();
        let planned_set_id = create_planned_set(&mut conn);
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let set = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            Some(planned_set_id),
        )
        .expect("logged set creation should succeed");

        assert_eq!(set.planned_set_id(), Some(planned_set_id));
    }

    #[test]
    fn create_set_with_nonexistent_planned_set_returns_error() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let result = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            Some(9999),
        );

        assert!(matches!(
            result,
            Err(LoggedSetError::AssociatedPlannedSetNotFound { id: 9999 })
        ));
    }

    #[test]
    fn sets_in_same_logged_exercise_get_sequential_positions() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);

        let first = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        )
        .expect("creation should succeed");
        let second = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        )
        .expect("creation should succeed");

        assert_eq!(first.position(), 0);
        assert_eq!(second.position(), 1);
    }

    #[test]
    fn get_set_returns_none_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = get(&conn, 9999).expect("query should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn list_returns_sets_for_a_logged_exercise_in_position_order() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);
        let first = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        )
        .expect("creation should succeed");
        let second = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            4,
            SetType::Regular { effort: None },
            None,
        )
        .expect("creation should succeed");

        let result = list(&conn, logged_exercise_id).expect("listing should succeed");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id(), first.id());
        assert_eq!(result[1].id(), second.id());
    }

    #[test]
    fn list_returns_error_when_logged_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = list(&conn, 9999);

        assert!(matches!(
            result,
            Err(LoggedSetError::AssociatedLoggedExerciseNotFound { id: 9999 })
        ));
    }

    #[test]
    fn deleting_a_logged_exercise_cascades_to_its_sets() {
        let mut conn = setup_test_db();
        let logged_exercise_id = create_weighted_logged_exercise(&conn);
        let set = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            None,
        )
        .expect("creation should succeed");

        logged_exercises::delete(&conn, logged_exercise_id)
            .expect("logged exercise deletion should succeed");

        assert!(
            get(&conn, set.id())
                .expect("query should succeed")
                .is_none()
        );
    }

    #[test]
    fn deleting_a_planned_set_nulls_the_link_but_keeps_the_logged_set() {
        let mut conn = setup_test_db();
        let planned_set_id = create_planned_set(&mut conn);
        let logged_exercise_id = create_weighted_logged_exercise(&conn);
        let set = create(
            &mut conn,
            logged_exercise_id,
            weighted(100.0),
            5,
            SetType::Regular { effort: None },
            Some(planned_set_id),
        )
        .expect("creation should succeed");

        sets::delete(&conn, planned_set_id).expect("planned set deletion should succeed");

        let persisted = get(&conn, set.id())
            .expect("query should succeed")
            .expect("logged set should still exist");
        assert_eq!(persisted.planned_set_id(), None);
    }

    #[test]
    fn delete_returns_not_found_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 9999);

        assert!(matches!(result, Err(LoggedSetError::NotFound { id: 9999 })));
    }
}
