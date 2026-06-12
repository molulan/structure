use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::PlannedExercise;
use crate::persistence::library_exercises::get_library_exercise;

#[derive(Debug, thiserror::Error)]
pub enum PlannedExerciseError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated workout {id} not found")]
    AssociatedWorkoutNotFound { id: i64 },
    #[error("associated exercise {id} not found")]
    AssociatedExerciseNotFound { id: i64 },
    #[error("planned exercise {id} not found")]
    NotFound { id: i64 },
    #[error("reorder list does not match the planned exercises of workout {workout_id}")]
    ReorderMismatch { workout_id: i64 },
}

pub(super) fn create_planned_exercises_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS planned_exercises (
            id INTEGER PRIMARY KEY,
            workout_id INTEGER NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
            library_exercise_id INTEGER NOT NULL REFERENCES library_exercises(id),
            position INTEGER NOT NULL,
            UNIQUE(workout_id, position)
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

pub fn create_planned_exercise(
    conn: &Connection,
    workout_id: i64,
    library_exercise_id: i64,
) -> Result<PlannedExercise, PlannedExerciseError> {
    if !workout_exists(conn, workout_id)? {
        return Err(PlannedExerciseError::AssociatedWorkoutNotFound { id: workout_id });
    }

    match get_library_exercise(conn, library_exercise_id)? {
        None => Err(PlannedExerciseError::AssociatedExerciseNotFound {
            id: library_exercise_id,
        }),
        Some(exercise) => {
            let next_position: i64 = conn.query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM planned_exercises WHERE workout_id = ?1",
                [workout_id],
                |row| row.get(0),
            )?;

            let position = u32::try_from(next_position)
                .expect("positions are non-negative and no workout will have 4 billion exercises");

            conn.execute(
                "INSERT INTO planned_exercises (workout_id, library_exercise_id, position) VALUES (?1, ?2, ?3)",
                params![workout_id, library_exercise_id, position],
            )?;

            let id = conn.last_insert_rowid();

            Ok(PlannedExercise::new(id, exercise, position))
        }
    }
}

pub fn get_planned_exercise(
    conn: &Connection,
    id: i64,
) -> Result<Option<PlannedExercise>, PlannedExerciseError> {
    let row = conn
        .query_row(
            "SELECT id, library_exercise_id, position FROM planned_exercises WHERE id = ?1",
            [id],
            |row| {
                let id: i64 = row.get(0)?;
                let library_exercise_id: i64 = row.get(1)?;
                let position: i64 = row.get(2)?;
                Ok((id, library_exercise_id, position))
            },
        )
        .optional()?;

    match row {
        None => Ok(None),
        Some((id, library_exercise_id, position)) => {
            let position =
                u32::try_from(position).expect("position stored in DB was originally a u32");
            let exercise = get_library_exercise(conn, library_exercise_id)?
                .expect("exercise FK in planned_exercises points to nonexistent exercise");

            Ok(Some(PlannedExercise::new(id, exercise, position)))
        }
    }
}

pub fn list_planned_exercises(
    conn: &Connection,
    workout_id: i64,
) -> Result<Vec<PlannedExercise>, PlannedExerciseError> {
    if !workout_exists(conn, workout_id)? {
        return Err(PlannedExerciseError::AssociatedWorkoutNotFound { id: workout_id });
    }

    let mut stmt = conn.prepare(
        "SELECT id, library_exercise_id, position FROM planned_exercises WHERE workout_id = ?1 ORDER BY position ASC",
    )?;

    let rows = stmt.query_map([workout_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    let mut planned_exercises = Vec::new();

    for row in rows {
        let (id, library_exercise_id, position): (i64, i64, i64) = row?;
        let position = u32::try_from(position).expect("position stored in DB was originally a u32");
        let exercise = get_library_exercise(conn, library_exercise_id)?.expect(
            "exercise FK in planned_exercises points to nonexistent exercise — data corrupted",
        );
        planned_exercises.push(PlannedExercise::new(id, exercise, position));
    }

    Ok(planned_exercises)
}

pub fn delete_planned_exercise(conn: &Connection, id: i64) -> Result<(), PlannedExerciseError> {
    let deleted = conn.execute("DELETE FROM planned_exercises WHERE id = ?1", [id])?;

    if deleted == 0 {
        return Err(PlannedExerciseError::NotFound { id });
    }

    Ok(())
}

pub fn reorder_planned_exercises(
    conn: &mut Connection,
    workout_id: i64,
    ordered_ids: &[i64],
) -> Result<(), PlannedExerciseError> {
    let matched = super::positions::reorder(
        conn,
        "planned_exercises",
        "workout_id",
        workout_id,
        ordered_ids,
    )?;

    if matched {
        Ok(())
    } else {
        Err(PlannedExerciseError::ReorderMismatch { workout_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::{ExerciseType, LibraryExercise, MesocycleMode, Workout},
        persistence::{
            connection,
            library_exercises::create_library_exercise,
            mesocycles::create_mesocycle,
            microcycles::create_microcycle,
            workouts::{create_workout, delete_workout},
        },
    };

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("Failed to create test database")
    }

    fn create_test_workout(conn: &Connection) -> Workout {
        let mesocycle = create_mesocycle(conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");

        let microcycle =
            create_microcycle(conn, mesocycle.id()).expect("microcycle creation should succeed");
        create_workout(conn, microcycle.id(), "Test Workout")
            .expect("workout creation should succeed")
    }

    fn create_test_exercise(conn: &Connection) -> LibraryExercise {
        create_library_exercise(conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed")
    }

    #[test]
    fn create_planned_exercise_for_nonexistent_workout_returns_error() {
        let conn = setup_test_db();
        let exercise = create_test_exercise(&conn);
        let result = create_planned_exercise(&conn, 9999, exercise.id());
        assert!(matches!(
            result,
            Err(PlannedExerciseError::AssociatedWorkoutNotFound { .. })
        ));
    }
    #[test]
    fn create_planned_exercise_for_nonexistent_exercise_returns_error() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let result = create_planned_exercise(&conn, workout.id(), 9999);
        assert!(matches!(
            result,
            Err(PlannedExerciseError::AssociatedExerciseNotFound { .. })
        ));
    }
    #[test]
    fn first_planned_exercise_in_workout_gets_position_0() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise = create_test_exercise(&conn);
        let planned = create_planned_exercise(&conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");

        assert_eq!(planned.position(), 0);
    }
    #[test]
    fn multiple_planned_exercises_in_same_workout_get_sequential_positions() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise_1 = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");
        let exercise_3 = create_library_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("third exercise creation should succeed");
        let planned_1 = create_planned_exercise(&conn, workout.id(), exercise_1.id())
            .expect("first planned exercise creation should succeed");
        let planned_2 = create_planned_exercise(&conn, workout.id(), exercise_2.id())
            .expect("second planned exercise creation should succeed");
        let planned_3 = create_planned_exercise(&conn, workout.id(), exercise_3.id())
            .expect("third planned exercise creation should succeed");

        assert_eq!(planned_1.position(), 0);
        assert_eq!(planned_2.position(), 1);
        assert_eq!(planned_3.position(), 2);
    }
    #[test]
    fn multiple_planned_exercises_in_same_workout_get_unique_ids() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise_1 = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");
        let planned_1 = create_planned_exercise(&conn, workout.id(), exercise_1.id())
            .expect("first planned exercise creation should succeed");
        let planned_2 = create_planned_exercise(&conn, workout.id(), exercise_2.id())
            .expect("second planned exercise creation should succeed");

        assert_ne!(planned_1.id(), planned_2.id());
    }

    #[test]
    fn get_planned_exercise_returns_none_for_invalid_id() {
        let conn = setup_test_db();
        let result = get_planned_exercise(&conn, 9999).expect("DB query should not fail");
        assert!(result.is_none());
    }
    #[test]
    fn get_planned_exercise_returns_correct_planned_exercise() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise_1 = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");
        let _ = create_planned_exercise(&conn, workout.id(), exercise_1.id())
            .expect("first planned exercise creation should succeed");
        let target = create_planned_exercise(&conn, workout.id(), exercise_2.id())
            .expect("second planned exercise creation should succeed");

        let result = get_planned_exercise(&conn, target.id())
            .expect("DB query should not fail")
            .expect("planned exercise should exist");

        assert_eq!(result.id(), target.id());
        assert_eq!(result.name(), target.name());
        assert_eq!(
            result.exercise().exercise_type(),
            target.exercise().exercise_type()
        );
        assert_eq!(result.position(), target.position());
    }

    #[test]
    fn list_planned_exercises_returns_empty_list_for_workout_with_no_planned_exercises() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);

        let result = list_planned_exercises(&conn, workout.id())
            .expect("listing planned exercises for a existing workout id should succeed");

        assert!(result.is_empty());
    }
    #[test]
    fn list_planned_exercises_returns_error_when_workout_does_not_exist() {
        let conn = setup_test_db();
        let result = list_planned_exercises(&conn, 9999);
        assert!(matches!(
            result,
            Err(PlannedExerciseError::AssociatedWorkoutNotFound { .. })
        ));
    }
    #[test]
    fn list_planned_exercises_returns_all_planned_exercises_for_a_specific_workout() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "Arms, Arms, Arms", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");

        let microcycle =
            create_microcycle(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        let target_workout = create_workout(&conn, microcycle.id(), "Arms & Arms")
            .expect("workout creation should succeed");

        let workout_2 = create_workout(&conn, microcycle.id(), "legs..")
            .expect("workout creation should succeed");

        let exercise_1 =
            create_library_exercise(&conn, "Arnolds Favorite Armblaster", ExerciseType::Weighted)
                .expect("exercise creation should succeed");

        let exercise_2 = create_library_exercise(
            &conn,
            "Arnolds Second Favorite Armblaster",
            ExerciseType::Bodyweight,
        )
        .expect("exercise creation should succeed");

        let exercise_3 = create_library_exercise(&conn, "squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let planned_exercise_1 =
            create_planned_exercise(&conn, target_workout.id(), exercise_1.id())
                .expect("planned_exercise creation should succeed");

        let planned_exercise_2 =
            create_planned_exercise(&conn, target_workout.id(), exercise_2.id())
                .expect("planned_exercise creation should succeed");

        let _ = create_planned_exercise(&conn, workout_2.id(), exercise_3.id())
            .expect("planned_exercise creation should succeed");

        let result = list_planned_exercises(&conn, target_workout.id())
            .expect("listing planned exercises should succeed");

        assert_eq!(2, result.len());
        assert_eq!(planned_exercise_1, result[0]);
        assert_eq!(planned_exercise_2, result[1]);
    }

    /// Returns the workout id and three planned exercises (positions 0, 1, 2),
    /// all referencing the same library exercise.
    fn workout_with_three_planned_exercises(
        conn: &Connection,
    ) -> (i64, PlannedExercise, PlannedExercise, PlannedExercise) {
        let workout = create_test_workout(conn);
        let exercise = create_test_exercise(conn);
        let a = create_planned_exercise(conn, workout.id(), exercise.id())
            .expect("creation should succeed");
        let b = create_planned_exercise(conn, workout.id(), exercise.id())
            .expect("creation should succeed");
        let c = create_planned_exercise(conn, workout.id(), exercise.id())
            .expect("creation should succeed");
        (workout.id(), a, b, c)
    }

    #[test]
    fn create_planned_exercise_after_delete_does_not_reuse_a_position() {
        let conn = setup_test_db();
        let (workout_id, _a, middle, _c) = workout_with_three_planned_exercises(&conn);

        delete_planned_exercise(&conn, middle.id()).expect("delete should succeed");

        let exercise = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        let next = create_planned_exercise(&conn, workout_id, exercise.id())
            .expect("creation should succeed");
        assert_eq!(next.position(), 3);
    }

    #[test]
    fn delete_planned_exercise_removes_it() {
        let conn = setup_test_db();
        let (_workout_id, planned, _b, _c) = workout_with_three_planned_exercises(&conn);

        delete_planned_exercise(&conn, planned.id()).expect("delete should succeed");

        let result = get_planned_exercise(&conn, planned.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn delete_planned_exercise_returns_not_found_when_it_does_not_exist() {
        let conn = setup_test_db();

        let result = delete_planned_exercise(&conn, 9999);

        assert!(matches!(
            result,
            Err(PlannedExerciseError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn delete_workout_cascades_to_its_planned_exercises() {
        let conn = setup_test_db();
        let (workout_id, planned, _b, _c) = workout_with_three_planned_exercises(&conn);

        delete_workout(&conn, workout_id).expect("delete should succeed");

        let result = get_planned_exercise(&conn, planned.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn reorder_planned_exercises_rewrites_positions_in_the_given_order() {
        let mut conn = setup_test_db();
        let (workout_id, a, b, c) = workout_with_three_planned_exercises(&conn);

        reorder_planned_exercises(&mut conn, workout_id, &[c.id(), a.id(), b.id()])
            .expect("reorder should succeed");

        let ordered = list_planned_exercises(&conn, workout_id).expect("listing should succeed");
        let ids: Vec<i64> = ordered.iter().map(|p| p.id()).collect();
        assert_eq!(ids, vec![c.id(), a.id(), b.id()]);
        assert_eq!(ordered[0].position(), 0);
        assert_eq!(ordered[1].position(), 1);
        assert_eq!(ordered[2].position(), 2);
    }

    #[test]
    fn reorder_planned_exercises_returns_mismatch_when_ids_do_not_match_children() {
        let mut conn = setup_test_db();
        let (workout_id, a, _b, _c) = workout_with_three_planned_exercises(&conn);

        let result = reorder_planned_exercises(&mut conn, workout_id, &[a.id()]);

        assert!(matches!(
            result,
            Err(PlannedExerciseError::ReorderMismatch { .. })
        ));
    }
}
