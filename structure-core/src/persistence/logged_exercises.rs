use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::tracking::LoggedExercise;
use crate::persistence::library_exercises;

#[derive(Debug, thiserror::Error)]
pub enum LoggedExerciseError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated logged session {id} not found")]
    AssociatedSessionNotFound { id: i64 },
    #[error("associated library exercise {id} not found")]
    AssociatedLibraryExerciseNotFound { id: i64 },
    #[error("associated planned exercise {id} not found")]
    AssociatedPlannedExerciseNotFound { id: i64 },
    #[error("logged exercise {id} not found")]
    NotFound { id: i64 },
}

pub(super) fn create_logged_exercises_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logged_exercises (
            id INTEGER PRIMARY KEY,
            logged_session_id INTEGER NOT NULL REFERENCES logged_sessions(id) ON DELETE CASCADE,
            planned_exercise_id INTEGER REFERENCES planned_exercises(id) ON DELETE SET NULL,
            library_exercise_id INTEGER NOT NULL REFERENCES library_exercises(id),
            position INTEGER NOT NULL,
            note TEXT,
            UNIQUE(logged_session_id, position)
        )",
        (),
    )?;
    Ok(())
}

fn logged_session_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM logged_sessions WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn planned_exercise_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM planned_exercises WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create(
    conn: &Connection,
    logged_session_id: i64,
    library_exercise_id: i64,
    planned_exercise_id: Option<i64>,
    note: Option<&str>,
) -> Result<LoggedExercise, LoggedExerciseError> {
    if !logged_session_exists(conn, logged_session_id)? {
        return Err(LoggedExerciseError::AssociatedSessionNotFound {
            id: logged_session_id,
        });
    }

    let Some(exercise) = library_exercises::get(conn, library_exercise_id)? else {
        return Err(LoggedExerciseError::AssociatedLibraryExerciseNotFound {
            id: library_exercise_id,
        });
    };

    if let Some(planned_id) = planned_exercise_id
        && !planned_exercise_exists(conn, planned_id)?
    {
        return Err(LoggedExerciseError::AssociatedPlannedExerciseNotFound { id: planned_id });
    }

    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM logged_exercises WHERE logged_session_id = ?1",
        [logged_session_id],
        |row| row.get(0),
    )?;
    let position = u32::try_from(next_position)
        .expect("positions are non-negative and no session will have 4 billion exercises");

    conn.execute(
        "INSERT INTO logged_exercises
            (logged_session_id, planned_exercise_id, library_exercise_id, position, note)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            logged_session_id,
            planned_exercise_id,
            library_exercise_id,
            position,
            note,
        ],
    )?;

    let id = conn.last_insert_rowid();

    Ok(LoggedExercise::new(
        id,
        exercise,
        position,
        planned_exercise_id,
        note.map(str::to_string),
    ))
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<LoggedExercise>, LoggedExerciseError> {
    let row = conn
        .query_row(
            "SELECT id, library_exercise_id, position, planned_exercise_id, note
             FROM logged_exercises WHERE id = ?1",
            [id],
            row_to_parts,
        )
        .optional()?;

    match row {
        None => Ok(None),
        Some(parts) => Ok(Some(build(conn, parts)?)),
    }
}

pub fn list(
    conn: &Connection,
    logged_session_id: i64,
) -> Result<Vec<LoggedExercise>, LoggedExerciseError> {
    if !logged_session_exists(conn, logged_session_id)? {
        return Err(LoggedExerciseError::AssociatedSessionNotFound {
            id: logged_session_id,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT id, library_exercise_id, position, planned_exercise_id, note
         FROM logged_exercises WHERE logged_session_id = ?1 ORDER BY position ASC",
    )?;
    let rows = stmt.query_map([logged_session_id], row_to_parts)?;

    let mut exercises = Vec::new();
    for row in rows {
        exercises.push(build(conn, row?)?);
    }
    Ok(exercises)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), LoggedExerciseError> {
    let deleted = conn.execute("DELETE FROM logged_exercises WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(LoggedExerciseError::NotFound { id });
    }
    Ok(())
}

/// The raw columns of a `logged_exercises` row, before the referenced library
/// exercise is fetched and embedded.
struct Parts {
    id: i64,
    library_exercise_id: i64,
    position: i64,
    planned_exercise_id: Option<i64>,
    note: Option<String>,
}

fn row_to_parts(row: &rusqlite::Row<'_>) -> rusqlite::Result<Parts> {
    Ok(Parts {
        id: row.get(0)?,
        library_exercise_id: row.get(1)?,
        position: row.get(2)?,
        planned_exercise_id: row.get(3)?,
        note: row.get(4)?,
    })
}

fn build(conn: &Connection, parts: Parts) -> Result<LoggedExercise, LoggedExerciseError> {
    let position =
        u32::try_from(parts.position).expect("position stored in DB was originally a u32");
    let exercise = library_exercises::get(conn, parts.library_exercise_id)?
        .expect("exercise FK in logged_exercises points to nonexistent exercise — data corrupted");

    Ok(LoggedExercise::new(
        parts.id,
        exercise,
        position,
        parts.planned_exercise_id,
        parts.note,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::planning::{ExerciseType, LibraryExercise, MesocycleMode, Workout};
    use crate::persistence::{
        connection, library_exercises, logged_sessions, mesocycles, microcycles, planned_exercises,
        workouts,
    };

    const STARTED: &str = "2026-06-26T10:00:00Z";

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("failed to create test database")
    }

    fn create_test_workout(conn: &Connection) -> Workout {
        let mesocycle = mesocycles::create(conn, "Test Mesocycle", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let microcycle =
            microcycles::create(conn, mesocycle.id()).expect("microcycle creation should succeed");
        workouts::create(conn, microcycle.id(), "Push").expect("workout creation should succeed")
    }

    fn create_test_exercise(conn: &Connection) -> LibraryExercise {
        library_exercises::create(conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed")
    }

    fn create_test_session(conn: &Connection) -> i64 {
        logged_sessions::create(conn, STARTED, None, None, None)
            .expect("session creation should succeed")
            .id()
    }

    #[test]
    fn create_unplanned_exercise_has_no_plan_link_and_embeds_the_exercise() {
        let conn = setup_test_db();
        let session_id = create_test_session(&conn);
        let exercise = create_test_exercise(&conn);

        let logged = create(&conn, session_id, exercise.id(), None, None)
            .expect("logged exercise creation should succeed");

        assert_eq!(logged.position(), 0);
        assert_eq!(logged.planned_exercise_id(), None);
        assert_eq!(logged.name(), "Bench Press");
        assert_eq!(logged.exercise().id(), exercise.id());
    }

    #[test]
    fn create_exercise_linked_to_a_planned_exercise_stores_the_link() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise = create_test_exercise(&conn);
        let planned = planned_exercises::create(&conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");
        let session_id = create_test_session(&conn);

        let logged = create(
            &conn,
            session_id,
            exercise.id(),
            Some(planned.id()),
            Some("paused reps"),
        )
        .expect("logged exercise creation should succeed");

        assert_eq!(logged.planned_exercise_id(), Some(planned.id()));
        assert_eq!(logged.note(), Some("paused reps"));
    }

    #[test]
    fn create_exercise_for_nonexistent_session_returns_error() {
        let conn = setup_test_db();
        let exercise = create_test_exercise(&conn);

        let result = create(&conn, 9999, exercise.id(), None, None);

        assert!(matches!(
            result,
            Err(LoggedExerciseError::AssociatedSessionNotFound { id: 9999 })
        ));
    }

    #[test]
    fn create_exercise_with_nonexistent_library_exercise_returns_error() {
        let conn = setup_test_db();
        let session_id = create_test_session(&conn);

        let result = create(&conn, session_id, 9999, None, None);

        assert!(matches!(
            result,
            Err(LoggedExerciseError::AssociatedLibraryExerciseNotFound { id: 9999 })
        ));
    }

    #[test]
    fn create_exercise_with_nonexistent_planned_exercise_returns_error() {
        let conn = setup_test_db();
        let session_id = create_test_session(&conn);
        let exercise = create_test_exercise(&conn);

        let result = create(&conn, session_id, exercise.id(), Some(9999), None);

        assert!(matches!(
            result,
            Err(LoggedExerciseError::AssociatedPlannedExerciseNotFound { id: 9999 })
        ));
    }

    #[test]
    fn exercises_in_same_session_get_sequential_positions() {
        let conn = setup_test_db();
        let session_id = create_test_session(&conn);
        let bench = library_exercises::create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let row = library_exercises::create(&conn, "Row", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let first =
            create(&conn, session_id, bench.id(), None, None).expect("creation should succeed");
        let second =
            create(&conn, session_id, row.id(), None, None).expect("creation should succeed");

        assert_eq!(first.position(), 0);
        assert_eq!(second.position(), 1);
    }

    #[test]
    fn get_exercise_returns_none_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = get(&conn, 9999).expect("query should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn list_returns_exercises_for_a_session_in_position_order() {
        let conn = setup_test_db();
        let session_id = create_test_session(&conn);
        let bench = library_exercises::create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let row = library_exercises::create(&conn, "Row", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let first =
            create(&conn, session_id, bench.id(), None, None).expect("creation should succeed");
        let second =
            create(&conn, session_id, row.id(), None, None).expect("creation should succeed");

        let result = list(&conn, session_id).expect("listing should succeed");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id(), first.id());
        assert_eq!(result[1].id(), second.id());
    }

    #[test]
    fn list_returns_error_when_session_does_not_exist() {
        let conn = setup_test_db();

        let result = list(&conn, 9999);

        assert!(matches!(
            result,
            Err(LoggedExerciseError::AssociatedSessionNotFound { id: 9999 })
        ));
    }

    #[test]
    fn deleting_a_session_cascades_to_its_logged_exercises() {
        let conn = setup_test_db();
        let session_id = create_test_session(&conn);
        let exercise = create_test_exercise(&conn);
        let logged =
            create(&conn, session_id, exercise.id(), None, None).expect("creation should succeed");

        logged_sessions::delete(&conn, session_id).expect("session deletion should succeed");

        assert!(
            get(&conn, logged.id())
                .expect("query should succeed")
                .is_none()
        );
    }

    #[test]
    fn deleting_a_planned_exercise_nulls_the_link_but_keeps_the_logged_exercise() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise = create_test_exercise(&conn);
        let planned = planned_exercises::create(&conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");
        let session_id = create_test_session(&conn);
        let logged = create(&conn, session_id, exercise.id(), Some(planned.id()), None)
            .expect("creation should succeed");

        planned_exercises::delete(&conn, planned.id()).expect("planned deletion should succeed");

        let persisted = get(&conn, logged.id())
            .expect("query should succeed")
            .expect("logged exercise should still exist");
        assert_eq!(persisted.planned_exercise_id(), None);
    }

    #[test]
    fn delete_returns_not_found_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 9999);

        assert!(matches!(
            result,
            Err(LoggedExerciseError::NotFound { id: 9999 })
        ));
    }
}
