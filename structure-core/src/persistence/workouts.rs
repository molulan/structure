use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::{Name, NameError, Workout};

#[derive(Debug, thiserror::Error)]
pub enum WorkoutError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated mesocycle {id} not found")]
    AssociatedMesocycleNotFound { id: i64 },
    #[error("workout {id} not found")]
    NotFound { id: i64 },
    #[error("reorder list does not match the workouts of mesocycle {mesocycle_id}")]
    ReorderMismatch { mesocycle_id: i64 },
    #[error(transparent)]
    InvalidName(#[from] NameError),
}

pub(super) fn create_workouts_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workouts (
            id INTEGER PRIMARY KEY,
            mesocycle_id INTEGER NOT NULL REFERENCES mesocycles(id) ON DELETE CASCADE,
            name TEXT NOT NULL CHECK(length(name) > 0),
            position INTEGER NOT NULL,
            UNIQUE(mesocycle_id, position)
        )",
        (),
    )?;
    Ok(())
}

fn mesocycle_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM mesocycles WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create(conn: &Connection, mesocycle_id: i64, name: &str) -> Result<Workout, WorkoutError> {
    let name = Name::new(name)?;

    if !mesocycle_exists(conn, mesocycle_id)? {
        return Err(WorkoutError::AssociatedMesocycleNotFound { id: mesocycle_id });
    }

    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM workouts WHERE mesocycle_id = ?1",
        [mesocycle_id],
        |row| row.get(0),
    )?;

    let position = u32::try_from(next_position)
        .expect("positions are non-negative and no mesocycle will have 4 billion workouts");

    conn.execute(
        "INSERT INTO workouts (mesocycle_id, name, position) VALUES (?1, ?2, ?3)",
        params![mesocycle_id, name.as_str(), position],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Workout::new(id, name, position))
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Workout>> {
    conn.query_row(
        "SELECT id, name, position FROM workouts WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let position: i64 = row.get(2)?;
            let position =
                u32::try_from(position).expect("position stored in DB was originally a u32");
            let name = Name::new(name).expect("name stored in the database was validated on write");
            Ok(Workout::new(id, name, position))
        },
    )
    .optional()
}

pub fn list(conn: &Connection, mesocycle_id: i64) -> Result<Vec<Workout>, WorkoutError> {
    if !mesocycle_exists(conn, mesocycle_id)? {
        return Err(WorkoutError::AssociatedMesocycleNotFound { id: mesocycle_id });
    }

    let mut stmt = conn.prepare(
        "SELECT id, name, position FROM workouts WHERE mesocycle_id = ?1 ORDER BY position ASC",
    )?;

    stmt.query_map([mesocycle_id], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        let position: i64 = row.get(2)?;
        let position = u32::try_from(position).expect("position stored in DB was originally a u32");
        let name = Name::new(name).expect("name stored in the database was validated on write");
        Ok(Workout::new(id, name, position))
    })?
    .map(|result| result.map_err(WorkoutError::from))
    .collect()
}

pub fn update(conn: &Connection, id: i64, name: &str) -> Result<Workout, WorkoutError> {
    let name = Name::new(name)?;

    let updated = conn.execute(
        "UPDATE workouts SET name = ?1 WHERE id = ?2",
        params![name.as_str(), id],
    )?;

    if updated == 0 {
        return Err(WorkoutError::NotFound { id });
    }

    let workout = get(conn, id)?.expect("workout exists immediately after a successful update");
    Ok(workout)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), WorkoutError> {
    let deleted = conn.execute("DELETE FROM workouts WHERE id = ?1", [id])?;

    if deleted == 0 {
        return Err(WorkoutError::NotFound { id });
    }

    Ok(())
}

pub fn reorder(
    conn: &mut Connection,
    mesocycle_id: i64,
    ordered_ids: &[i64],
) -> Result<(), WorkoutError> {
    let matched = super::positions::reorder(
        conn,
        "workouts",
        &[("mesocycle_id", mesocycle_id)],
        ordered_ids,
    )?;

    if matched {
        Ok(())
    } else {
        Err(WorkoutError::ReorderMismatch { mesocycle_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::MesocycleMode,
        persistence::{connection, mesocycles},
    };

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn get_workout_returns_none_on_invalid_id() {
        let conn = setup_test_db();

        let result = get(&conn, 1234).expect("Should return None");

        assert!(result.is_none());
    }

    #[test]
    fn get_workout_returns_correct_workout() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle_1 = mesocycles::create(&conn, "small arms", mode)
            .expect("Should be able to create mesoocycle");
        let mesocycle_2 = mesocycles::create(&conn, "BIG ARMS", mode)
            .expect("Should be able to create mesoocycle");

        let _ = create(&conn, mesocycle_1.id(), "Calfs").expect("Should be able to create workout");
        let target = create(&conn, mesocycle_2.id(), "Triceps And Biceps")
            .expect("Should be able to create workout");

        let result = get(&conn, target.id())
            .expect("DB query should not fail")
            .expect("workout should exist");

        assert_eq!(target, result);
    }

    #[test]
    fn list_workouts_returns_empty_list_for_mesocycle_with_no_workouts() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let result =
            list(&conn, mesocycle.id()).expect("listing workouts for a valid id should succeed");

        assert!(result.is_empty());
    }

    #[test]
    fn list_workouts_returns_error_when_called_with_invalid_mesocycle_id() {
        let conn = setup_test_db();

        let result = list(&conn, 1234);

        assert!(result.is_err());
    }

    #[test]
    fn create_workout_generates_workout_with_position_0_in_empty_mesocycle() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let workout =
            create(&conn, mesocycle.id(), "CHEST").expect("workout creation should succeed");

        assert_eq!(workout.position(), 0);
    }

    #[test]
    fn multiple_workouts_in_same_mesocycle_get_increasing_position_numbers() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let workout_1 =
            create(&conn, mesocycle.id(), "CHEST").expect("workout creation should succeed");
        let workout_2 =
            create(&conn, mesocycle.id(), "CHEST again").expect("workout creation should succeed");
        let workout_3 = create(&conn, mesocycle.id(), "CHEST forever")
            .expect("workout creation should succeed");

        assert_eq!(workout_1.position(), 0);
        assert_eq!(workout_2.position(), 1);
        assert_eq!(workout_3.position(), 2);
    }

    #[test]
    fn multiple_workouts_in_same_mesocycle_get_unique_ids() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let workout_1 =
            create(&conn, mesocycle.id(), "CHEST").expect("workout creation should succeed");
        let workout_2 =
            create(&conn, mesocycle.id(), "CHEST again").expect("workout creation should succeed");

        assert_ne!(workout_1.id(), workout_2.id());
    }

    #[test]
    fn created_workout_appears_in_list_with_correct_id_name_and_position() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let workout =
            create(&conn, mesocycle.id(), "CHEST").expect("workout creation should succeed");
        let result =
            list(&conn, mesocycle.id()).expect("listing workouts for a valid id should succeed");

        assert_eq!(result[0].id(), workout.id());
        assert_eq!(result[0].name(), workout.name());
        assert_eq!(result[0].position(), workout.position());
    }

    #[test]
    fn multiple_workouts_appear_in_list_with_correct_id_name_and_position() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let workout_1 =
            create(&conn, mesocycle.id(), "CHEST").expect("workout creation should succeed");
        let workout_2 =
            create(&conn, mesocycle.id(), "CHEST again").expect("workout creation should succeed");
        let result =
            list(&conn, mesocycle.id()).expect("listing workouts for a valid id should succeed");

        assert_eq!(result[0].id(), workout_1.id());
        assert_eq!(result[0].name(), workout_1.name());
        assert_eq!(result[0].position(), workout_1.position());

        assert_eq!(result[1].id(), workout_2.id());
        assert_eq!(result[1].name(), workout_2.name());
        assert_eq!(result[1].position(), workout_2.position());
    }

    #[test]
    fn workouts_are_scoped_to_their_parent_mesocycle() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle_1 = mesocycles::create(&conn, "small arms", mode)
            .expect("mesocycle creation should succeed");
        let workout_1 =
            create(&conn, mesocycle_1.id(), "CHEST").expect("workout creation should succeed");

        let mesocycle_2 =
            mesocycles::create(&conn, "BIG ARMS", mode).expect("mesocycle creation should succeed");
        let workout_2 = create(&conn, mesocycle_2.id(), "CHEST again")
            .expect("workout creation should succeed");

        let result_1 =
            list(&conn, mesocycle_1.id()).expect("listing workouts for a valid id should succeed");
        assert_eq!(result_1.len(), 1);
        assert_eq!(result_1[0], workout_1);

        let result_2 =
            list(&conn, mesocycle_2.id()).expect("listing workouts for a valid id should succeed");
        assert_eq!(result_2.len(), 1);
        assert_eq!(result_2[0], workout_2);
    }

    #[test]
    fn creating_workout_with_empty_name_returns_error() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "Pecosaurus Rex", mode)
            .expect("mesocycle creation should succeed");

        let result = create(&conn, mesocycle.id(), "");

        assert!(result.is_err());
    }

    #[test]
    fn creating_workout_with_invalid_mesocycle_id_returns_error() {
        let conn = setup_test_db();

        let result = create(&conn, 1234, "CHEST");

        assert!(result.is_err());
    }

    /// Returns the mesocycle id and its three workouts (positions 0, 1, 2).
    fn mesocycle_with_three_workouts(conn: &Connection) -> (i64, Workout, Workout, Workout) {
        let mesocycle = mesocycles::create(conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let a = create(conn, mesocycle.id(), "Push").expect("creation should succeed");
        let b = create(conn, mesocycle.id(), "Pull").expect("creation should succeed");
        let c = create(conn, mesocycle.id(), "Legs").expect("creation should succeed");
        (mesocycle.id(), a, b, c)
    }

    #[test]
    fn update_workout_changes_name_and_keeps_position() {
        let conn = setup_test_db();
        let (_mesocycle_id, _a, workout, _c) = mesocycle_with_three_workouts(&conn);

        let updated = update(&conn, workout.id(), "Upper").expect("update should succeed");

        assert_eq!(updated.name(), "Upper");
        assert_eq!(updated.position(), workout.position());

        let persisted = get(&conn, workout.id())
            .expect("query should succeed")
            .expect("workout should exist");
        assert_eq!(persisted.name(), "Upper");
    }

    #[test]
    fn update_workout_returns_not_found_when_workout_does_not_exist() {
        let conn = setup_test_db();

        let result = update(&conn, 1234, "Upper");

        assert!(matches!(result, Err(WorkoutError::NotFound { id: 1234 })));
    }

    #[test]
    fn create_workout_after_delete_does_not_reuse_a_position() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        let _first = create(&conn, mesocycle.id(), "Push").expect("creation should succeed");
        let middle = create(&conn, mesocycle.id(), "Pull").expect("creation should succeed");
        let _last = create(&conn, mesocycle.id(), "Legs").expect("creation should succeed");

        delete(&conn, middle.id()).expect("delete should succeed");

        let next = create(&conn, mesocycle.id(), "Arms").expect("creation should succeed");
        assert_eq!(next.position(), 3);
    }

    #[test]
    fn delete_workout_removes_it() {
        let conn = setup_test_db();
        let (_mesocycle_id, workout, _b, _c) = mesocycle_with_three_workouts(&conn);

        delete(&conn, workout.id()).expect("delete should succeed");

        let result = get(&conn, workout.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn delete_workout_returns_not_found_when_workout_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 1234);

        assert!(matches!(result, Err(WorkoutError::NotFound { id: 1234 })));
    }

    #[test]
    fn reorder_workouts_rewrites_positions_in_the_given_order() {
        let mut conn = setup_test_db();
        let (mesocycle_id, a, b, c) = mesocycle_with_three_workouts(&conn);

        reorder(&mut conn, mesocycle_id, &[c.id(), a.id(), b.id()])
            .expect("reorder should succeed");

        let ordered = list(&conn, mesocycle_id).expect("listing should succeed");
        let ids: Vec<i64> = ordered.iter().map(|w| w.id()).collect();
        assert_eq!(ids, vec![c.id(), a.id(), b.id()]);
        assert_eq!(ordered[0].position(), 0);
        assert_eq!(ordered[1].position(), 1);
        assert_eq!(ordered[2].position(), 2);
    }

    #[test]
    fn reorder_workouts_returns_mismatch_when_ids_do_not_match_children() {
        let mut conn = setup_test_db();
        let (mesocycle_id, a, _b, _c) = mesocycle_with_three_workouts(&conn);

        let result = reorder(&mut conn, mesocycle_id, &[a.id()]);

        assert!(matches!(result, Err(WorkoutError::ReorderMismatch { .. })));
    }
}
