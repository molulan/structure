use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::{Weight, WeightUnit};
use crate::domain::tracking::LoggedSession;

#[derive(Debug, thiserror::Error)]
pub enum LoggedSessionError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("planned workout {id} not found")]
    AssociatedWorkoutNotFound { id: i64 },
    #[error("logged session {id} not found")]
    NotFound { id: i64 },
}

pub(super) fn create_logged_sessions_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logged_sessions (
            id INTEGER PRIMARY KEY,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            bodyweight_value REAL,
            bodyweight_unit TEXT CHECK(bodyweight_unit IN ('Kg', 'Lbs')),
            note TEXT,
            planned_workout_id INTEGER REFERENCES workouts(id) ON DELETE SET NULL,
            CHECK((bodyweight_value IS NULL) = (bodyweight_unit IS NULL))
        )",
        (),
    )?;
    Ok(())
}

fn workout_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM workouts WHERE id = ?1", [id], |row| {
            row.get(0)
        })?;
    Ok(count > 0)
}

pub fn create(
    conn: &Connection,
    started_at: &str,
    planned_workout_id: Option<i64>,
    bodyweight: Option<Weight>,
    note: Option<&str>,
) -> Result<LoggedSession, LoggedSessionError> {
    if let Some(workout_id) = planned_workout_id
        && !workout_exists(conn, workout_id)?
    {
        return Err(LoggedSessionError::AssociatedWorkoutNotFound { id: workout_id });
    }

    let (bodyweight_value, bodyweight_unit) = match bodyweight {
        None => (None, None),
        Some(weight) => (
            Some(weight.value()),
            Some(weight_unit_to_str(weight.unit())),
        ),
    };

    conn.execute(
        "INSERT INTO logged_sessions
            (started_at, completed_at, bodyweight_value, bodyweight_unit, note, planned_workout_id)
         VALUES (?1, NULL, ?2, ?3, ?4, ?5)",
        params![
            started_at,
            bodyweight_value,
            bodyweight_unit,
            note,
            planned_workout_id,
        ],
    )?;

    let id = conn.last_insert_rowid();

    Ok(LoggedSession::new(
        id,
        started_at.to_string(),
        None,
        bodyweight,
        note.map(str::to_string),
        planned_workout_id,
    ))
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<LoggedSession>> {
    conn.query_row(
        "SELECT id, started_at, completed_at, bodyweight_value, bodyweight_unit, note, planned_workout_id
         FROM logged_sessions WHERE id = ?1",
        [id],
        row_to_logged_session,
    )
    .optional()
}

pub fn list(conn: &Connection) -> Result<Vec<LoggedSession>, LoggedSessionError> {
    list_where(conn, "")
}

pub fn list_in_progress(conn: &Connection) -> Result<Vec<LoggedSession>, LoggedSessionError> {
    list_where(conn, "WHERE completed_at IS NULL")
}

fn list_where(conn: &Connection, filter: &str) -> Result<Vec<LoggedSession>, LoggedSessionError> {
    let sql = format!(
        "SELECT id, started_at, completed_at, bodyweight_value, bodyweight_unit, note, planned_workout_id
         FROM logged_sessions {filter} ORDER BY started_at DESC, id DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_logged_session)?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }
    Ok(sessions)
}

pub fn finish(conn: &Connection, id: i64, completed_at: &str) -> Result<(), LoggedSessionError> {
    let updated = conn.execute(
        "UPDATE logged_sessions SET completed_at = ?1 WHERE id = ?2",
        params![completed_at, id],
    )?;
    if updated == 0 {
        return Err(LoggedSessionError::NotFound { id });
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), LoggedSessionError> {
    let deleted = conn.execute("DELETE FROM logged_sessions WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(LoggedSessionError::NotFound { id });
    }
    Ok(())
}

fn row_to_logged_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<LoggedSession> {
    let id = row.get(0)?;
    let started_at: String = row.get(1)?;
    let completed_at: Option<String> = row.get(2)?;
    let bodyweight_value: Option<f64> = row.get(3)?;
    let bodyweight_unit: Option<String> = row.get(4)?;
    let note: Option<String> = row.get(5)?;
    let planned_workout_id: Option<i64> = row.get(6)?;

    let bodyweight = bodyweight_value
        .zip(bodyweight_unit)
        .map(|(value, unit)| Weight::new(value, weight_unit_from_str(&unit)));

    Ok(LoggedSession::new(
        id,
        started_at,
        completed_at,
        bodyweight,
        note,
        planned_workout_id,
    ))
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
        other => panic!("unknown bodyweight_unit in DB: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::planning::{MesocycleMode, Workout};
    use crate::persistence::{connection, mesocycles, microcycles, workouts};

    const STARTED: &str = "2026-06-26T10:00:00Z";
    const COMPLETED: &str = "2026-06-26T11:30:00Z";

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

    #[test]
    fn create_ad_hoc_session_has_no_plan_link_and_is_in_progress() {
        let conn = setup_test_db();

        let session =
            create(&conn, STARTED, None, None, None).expect("session creation should succeed");

        assert_eq!(session.started_at(), STARTED);
        assert_eq!(session.completed_at(), None);
        assert_eq!(session.planned_workout_id(), None);
        assert_eq!(session.bodyweight(), None);
        assert_eq!(session.note(), None);
    }

    #[test]
    fn create_session_linked_to_a_planned_workout_stores_the_link() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);

        let session = create(
            &conn,
            STARTED,
            Some(workout.id()),
            None,
            Some("felt strong"),
        )
        .expect("session creation should succeed");

        assert_eq!(session.planned_workout_id(), Some(workout.id()));
        assert_eq!(session.note(), Some("felt strong"));
    }

    #[test]
    fn create_session_with_nonexistent_workout_returns_error() {
        let conn = setup_test_db();

        let result = create(&conn, STARTED, Some(9999), None, None);

        assert!(matches!(
            result,
            Err(LoggedSessionError::AssociatedWorkoutNotFound { id: 9999 })
        ));
    }

    #[test]
    fn create_session_with_bodyweight_round_trips_through_the_database() {
        let conn = setup_test_db();
        let bodyweight = Weight::new(82.5, WeightUnit::Kg);

        let session = create(&conn, STARTED, None, Some(bodyweight), None)
            .expect("session creation should succeed");

        let persisted = get(&conn, session.id())
            .expect("query should succeed")
            .expect("session should exist");
        assert_eq!(persisted.bodyweight(), Some(bodyweight));
    }

    #[test]
    fn get_session_returns_none_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = get(&conn, 9999).expect("query should succeed");

        assert!(result.is_none());
    }

    #[test]
    fn finish_sets_completed_at() {
        let conn = setup_test_db();
        let session =
            create(&conn, STARTED, None, None, None).expect("session creation should succeed");

        finish(&conn, session.id(), COMPLETED).expect("finishing should succeed");

        let persisted = get(&conn, session.id())
            .expect("query should succeed")
            .expect("session should exist");
        assert_eq!(persisted.completed_at(), Some(COMPLETED));
    }

    #[test]
    fn finish_returns_not_found_when_session_does_not_exist() {
        let conn = setup_test_db();

        let result = finish(&conn, 9999, COMPLETED);

        assert!(matches!(
            result,
            Err(LoggedSessionError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn list_in_progress_returns_only_unfinished_sessions() {
        let conn = setup_test_db();
        let finished =
            create(&conn, STARTED, None, None, None).expect("session creation should succeed");
        let in_progress =
            create(&conn, STARTED, None, None, None).expect("session creation should succeed");
        finish(&conn, finished.id(), COMPLETED).expect("finishing should succeed");

        let result = list_in_progress(&conn).expect("listing should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), in_progress.id());
    }

    #[test]
    fn list_returns_all_sessions() {
        let conn = setup_test_db();
        create(&conn, STARTED, None, None, None).expect("session creation should succeed");
        create(&conn, STARTED, None, None, None).expect("session creation should succeed");

        let result = list(&conn).expect("listing should succeed");

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn deleting_a_planned_workout_nulls_the_session_link() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let session = create(&conn, STARTED, Some(workout.id()), None, None)
            .expect("session creation should succeed");

        workouts::delete(&conn, workout.id()).expect("workout deletion should succeed");

        let persisted = get(&conn, session.id())
            .expect("query should succeed")
            .expect("session should still exist");
        assert_eq!(persisted.planned_workout_id(), None);
    }

    #[test]
    fn delete_session_removes_it() {
        let conn = setup_test_db();
        let session =
            create(&conn, STARTED, None, None, None).expect("session creation should succeed");

        delete(&conn, session.id()).expect("delete should succeed");

        assert!(
            get(&conn, session.id())
                .expect("query should succeed")
                .is_none()
        );
    }

    #[test]
    fn delete_session_returns_not_found_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 9999);

        assert!(matches!(
            result,
            Err(LoggedSessionError::NotFound { id: 9999 })
        ));
    }
}
