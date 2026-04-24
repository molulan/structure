use rusqlite::{Connection, OptionalExtension, params};

use crate::{domain::planning::Workout, errors::WorkoutError};

pub(super) fn create_workouts_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workouts (
            id INTEGER PRIMARY KEY,
            microcycle_id INTEGER NOT NULL REFERENCES microcycles(id),
            name TEXT NOT NULL CHECK(length(name) > 0),
            position INTEGER NOT NULL,
            UNIQUE(microcycle_id, position)
        )",
        (),
    )?;
    Ok(())
}

fn microcycle_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM microcycles WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create_workout(
    conn: &Connection,
    microcycle_id: i64,
    name: &str,
) -> Result<Workout, WorkoutError> {
    if !microcycle_exists(conn, microcycle_id)? {
        return Err(WorkoutError::AssociatedMicrocycleNotFound { id: microcycle_id });
    }

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM workouts WHERE microcycle_id = ?1",
        [microcycle_id],
        |row| row.get(0),
    )?;

    // COUNT(*) is always non-negative and well within u32 range for this domain
    let position = count as u32;

    conn.execute(
        "INSERT INTO workouts (microcycle_id, name, position) VALUES (?1, ?2, ?3)",
        params![microcycle_id, name, position],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Workout::new(id, name, position))
}

pub fn get_workout(conn: &Connection, id: i64) -> rusqlite::Result<Option<Workout>> {
    conn.query_row(
        "SELECT id, name, position FROM workouts WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let position: i64 = row.get(2)?;
            Ok(Workout::new(id, name, position as u32))
        },
    )
    .optional()
}

pub fn list_workouts(conn: &Connection, microcycle_id: i64) -> Result<Vec<Workout>, WorkoutError> {
    if !microcycle_exists(conn, microcycle_id)? {
        return Err(WorkoutError::AssociatedMicrocycleNotFound { id: microcycle_id });
    }

    let mut stmt = conn.prepare(
        "SELECT id, name, position FROM workouts WHERE microcycle_id = ?1 ORDER BY position ASC",
    )?;

    stmt.query_map([microcycle_id], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        let position: i64 = row.get(2)?;
        Ok(Workout::new(id, name, position as u32))
    })?
    .map(|result| result.map_err(WorkoutError::from))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{
        mesocycles::create_mesocycle, microcycles::create_microcycle, sqlite,
    };

    fn setup_test_db() -> Connection {
        sqlite::init_db(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn get_workout_returns_none_on_invalid_id() {
        let conn = setup_test_db();

        let result = get_workout(&conn, 1234).expect("Should return None");

        assert!(result.is_none());
    }

    #[test]
    fn get_workout_returns_correct_workout() {
        let conn = setup_test_db();
        let mesocycle_1 =
            create_mesocycle(&conn, "small arms").expect("Should be able to create mesoocycle");
        let mesocycle_2 =
            create_mesocycle(&conn, "BIG ARMS").expect("Should be able to create mesoocycle");

        let microcycle_1 = create_microcycle(&conn, mesocycle_1.id())
            .expect("Should be able to create microcycle");
        let microcycle_2 = create_microcycle(&conn, mesocycle_2.id())
            .expect("Should be able to create microcycle");

        let _ = create_workout(&conn, microcycle_1.id(), "Calfs")
            .expect("Should be able to create workout");
        let target = create_workout(&conn, microcycle_2.id(), "Triceps And Biceps")
            .expect("Should be able to create workout");

        let result = get_workout(&conn, target.id())
            .expect("DB query should not fail")
            .expect("workout should exist");

        assert_eq!(target, result);
    }

    #[test]
    fn list_workouts_returns_empty_list_for_microcycle_with_no_workouts() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let result = list_workouts(&conn, microcycle.id()).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn list_workouts_returns_error_when_called_with_invalid_microcycle_id() {
        let conn = setup_test_db();

        let result = list_workouts(&conn, 1234);

        assert!(result.is_err());
    }

    #[test]
    fn create_workout_generates_workout_with_position_0_in_empty_microcycle() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let workout = create_workout(&conn, microcycle.id(), "CHEST").unwrap();

        assert_eq!(workout.position(), 0);
    }

    #[test]
    fn multiple_workouts_in_same_microcycle_get_increasing_position_numbers() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let workout_1 = create_workout(&conn, microcycle.id(), "CHEST").unwrap();
        let workout_2 = create_workout(&conn, microcycle.id(), "CHEST again").unwrap();
        let workout_3 = create_workout(&conn, microcycle.id(), "CHEST forever").unwrap();

        assert_eq!(workout_1.position(), 0);
        assert_eq!(workout_2.position(), 1);
        assert_eq!(workout_3.position(), 2);
    }

    #[test]
    fn multiple_workouts_in_same_microcycle_get_unique_ids() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let workout_1 = create_workout(&conn, microcycle.id(), "CHEST").unwrap();
        let workout_2 = create_workout(&conn, microcycle.id(), "CHEST again").unwrap();

        assert_ne!(workout_1.id(), workout_2.id());
    }

    #[test]
    fn created_workout_appears_in_list_with_correct_id_name_and_position() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let workout = create_workout(&conn, microcycle.id(), "CHEST").unwrap();
        let result = list_workouts(&conn, microcycle.id()).unwrap();

        assert_eq!(result[0].id(), workout.id());
        assert_eq!(result[0].name(), workout.name());
        assert_eq!(result[0].position(), workout.position());
    }

    #[test]
    fn multiple_workouts_appear_in_list_with_correct_id_name_and_position() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let workout_1 = create_workout(&conn, microcycle.id(), "CHEST").unwrap();
        let workout_2 = create_workout(&conn, microcycle.id(), "CHEST again").unwrap();
        let result = list_workouts(&conn, microcycle.id()).unwrap();

        assert_eq!(result[0].id(), workout_1.id());
        assert_eq!(result[0].name(), workout_1.name());
        assert_eq!(result[0].position(), workout_1.position());

        assert_eq!(result[1].id(), workout_2.id());
        assert_eq!(result[1].name(), workout_2.name());
        assert_eq!(result[1].position(), workout_2.position());
    }

    #[test]
    fn workouts_are_scoped_to_their_parent_microcycle() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();

        let microcycle_1 = create_microcycle(&conn, mesocycle.id()).unwrap();
        let workout_1 = create_workout(&conn, microcycle_1.id(), "CHEST").unwrap();

        let microcycle_2 = create_microcycle(&conn, mesocycle.id()).unwrap();
        let workout_2 = create_workout(&conn, microcycle_2.id(), "CHEST again").unwrap();

        let result_1 = list_workouts(&conn, microcycle_1.id()).unwrap();
        assert_eq!(result_1.len(), 1);
        assert_eq!(result_1[0], workout_1);

        let result_2 = list_workouts(&conn, microcycle_2.id()).unwrap();
        assert_eq!(result_2.len(), 1);
        assert_eq!(result_2[0], workout_2);
    }

    #[test]
    fn creating_workout_with_empty_name_returns_error() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "Pecosaurus Rex").unwrap();
        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let result = create_workout(&conn, microcycle.id(), "");

        assert!(result.is_err());
    }

    #[test]
    fn creating_workout_with_invalid_microcycle_id_returns_error() {
        let conn = setup_test_db();

        let result = create_workout(&conn, 1234, "CHEST");

        assert!(result.is_err());
    }
}
