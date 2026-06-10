use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::{
    ExerciseType, LibraryExercise, PlannedExercise, PlannedExerciseValidationError,
};

#[derive(Debug, thiserror::Error)]
pub enum LibraryExerciseError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("exercise with name '{name}' already exists")]
    DuplicateName { name: String },
    #[error("exercise {id} not found")]
    NotFound { id: i64 },
    #[error("exercise {id} is used by one or more planned exercises and cannot be deleted")]
    InUse { id: i64 },
}

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
    #[error("set error: {0}")]
    SetOperation(#[from] super::sets::SetError),
    #[error("validation error: {0}")]
    ValidationError(#[from] PlannedExerciseValidationError),
}

pub(super) fn create_library_exercises_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS library_exercises (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL CHECK(length(name) > 0),
            exercise_type TEXT NOT NULL CHECK(
                exercise_type IN (
                    'Bodyweight', 'WeightedBodyweight', 'AssistedBodyweight', 'Weighted'
                )
            )
        )",
        (),
    )?;
    Ok(())
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

fn library_exercise_name_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM library_exercises WHERE name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn exercise_type_from_str(s: &str) -> ExerciseType {
    match s {
        "Bodyweight" => ExerciseType::Bodyweight,
        "WeightedBodyweight" => ExerciseType::WeightedBodyweight,
        "AssistedBodyweight" => ExerciseType::AssistedBodyweight,
        "Weighted" => ExerciseType::Weighted,
        other => panic!("Unknown exercise_type '{}'", other),
    }
}

pub fn create_library_exercise(
    conn: &Connection,
    name: &str,
    exercise_type: ExerciseType,
) -> Result<LibraryExercise, LibraryExerciseError> {
    if library_exercise_name_exists(conn, name)? {
        return Err(LibraryExerciseError::DuplicateName {
            name: name.to_string(),
        });
    }

    conn.execute(
        "INSERT INTO library_exercises (name, exercise_type) VALUES (?1, ?2)",
        params![name, exercise_type.to_string()],
    )?;

    let id = conn.last_insert_rowid();

    Ok(LibraryExercise::new(id, name, exercise_type))
}

pub fn get_library_exercise(
    conn: &Connection,
    id: i64,
) -> rusqlite::Result<Option<LibraryExercise>> {
    conn.query_row(
        "SELECT id, name, exercise_type FROM library_exercises WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let exercise_type: String = row.get(2)?;
            let exercise_type = exercise_type_from_str(exercise_type.as_str());
            Ok(LibraryExercise::new(id, name, exercise_type))
        },
    )
    .optional()
}

pub fn list_library_exercises(
    conn: &Connection,
) -> Result<Vec<LibraryExercise>, LibraryExerciseError> {
    let mut stmt =
        conn.prepare("SELECT id, name, exercise_type FROM library_exercises ORDER BY name ASC")?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

    let mut exercises = Vec::new();
    for row in rows {
        let (id, name, exercise_type): (i64, String, String) = row?;
        let exercise_type = exercise_type_from_str(exercise_type.as_str());
        exercises.push(LibraryExercise::new(id, name, exercise_type));
    }

    Ok(exercises)
}

pub fn update_library_exercise(
    conn: &Connection,
    id: i64,
    name: &str,
    exercise_type: ExerciseType,
) -> Result<LibraryExercise, LibraryExerciseError> {
    if get_library_exercise(conn, id)?.is_none() {
        return Err(LibraryExerciseError::NotFound { id });
    }

    let name_taken: i64 = conn.query_row(
        "SELECT COUNT(*) FROM library_exercises WHERE name = ?1 AND id != ?2",
        params![name, id],
        |row| row.get(0),
    )?;
    if name_taken > 0 {
        return Err(LibraryExerciseError::DuplicateName {
            name: name.to_string(),
        });
    }

    conn.execute(
        "UPDATE library_exercises SET name = ?1, exercise_type = ?2 WHERE id = ?3",
        params![name, exercise_type.to_string(), id],
    )?;

    Ok(LibraryExercise::new(id, name, exercise_type))
}

pub fn delete_library_exercise(conn: &Connection, id: i64) -> Result<(), LibraryExerciseError> {
    let in_use: i64 = conn.query_row(
        "SELECT COUNT(*) FROM planned_exercises WHERE library_exercise_id = ?1",
        [id],
        |row| row.get(0),
    )?;
    if in_use > 0 {
        return Err(LibraryExerciseError::InUse { id });
    }

    let deleted = conn.execute("DELETE FROM library_exercises WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(LibraryExerciseError::NotFound { id });
    }

    Ok(())
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

            Ok(PlannedExercise::new(id, exercise, position, Vec::new())
                .expect("newly created exercise has no sets, so validation cannot fail"))
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
            let sets = super::sets::list_planned_sets(conn, id)?;
            let planned_exercise = PlannedExercise::new(id, exercise, position, sets)?;

            Ok(Some(planned_exercise))
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
        let sets = super::sets::list_planned_sets(conn, id)?;
        let planned_exercise = PlannedExercise::new(id, exercise, position, sets)?;
        planned_exercises.push(planned_exercise);
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
        domain::planning::{MesocycleMode, Workout},
        persistence::{
            connection, mesocycles::create_mesocycle, microcycles::create_microcycle,
            workouts::create_workout,
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
    fn create_exercise_with_valid_name_and_type_succeeds() {
        let conn = setup_test_db();
        let exercise = create_library_exercise(&conn, "Squat", ExerciseType::Bodyweight)
            .expect("exercise creation should succeed");

        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.exercise_type(), ExerciseType::Bodyweight);
    }
    #[test]
    fn create_exercise_with_empty_name_returns_error() {
        let conn = setup_test_db();
        let result = create_library_exercise(&conn, "", ExerciseType::Weighted);
        assert!(result.is_err());
    }
    #[test]
    fn create_exercise_with_duplicate_name_returns_duplicate_name_error() {
        let conn = setup_test_db();
        create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");

        let result = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::DuplicateName { .. })
        ));
    }
    #[test]
    fn create_exercise_assigns_unique_ids_to_different_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_library_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        assert_ne!(exercise_1.id(), exercise_2.id());
    }
    #[test]
    fn all_four_exercise_types_can_be_created() {
        let conn = setup_test_db();
        let bodyweight = create_library_exercise(&conn, "Push Up", ExerciseType::Bodyweight)
            .expect("bodyweight exercise creation should succeed");
        let weighted_bodyweight =
            create_library_exercise(&conn, "Pull Up", ExerciseType::WeightedBodyweight)
                .expect("weighted bodyweight exercise creation should succeed");
        let assisted_bodyweight =
            create_library_exercise(&conn, "Assisted Pull Up", ExerciseType::AssistedBodyweight)
                .expect("assisted bodyweight exercise creation should succeed");
        let weighted = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("weighted exercise creation should succeed");

        assert_eq!(bodyweight.exercise_type(), ExerciseType::Bodyweight);
        assert_eq!(
            weighted_bodyweight.exercise_type(),
            ExerciseType::WeightedBodyweight
        );
        assert_eq!(
            assisted_bodyweight.exercise_type(),
            ExerciseType::AssistedBodyweight
        );
        assert_eq!(weighted.exercise_type(), ExerciseType::Weighted);
    }

    #[test]
    fn get_exercise_returns_none_when_exercise_does_not_exist() {
        let conn = setup_test_db();
        let result = get_library_exercise(&conn, 9999).expect("DB query should not fail");
        assert!(result.is_none());
    }
    #[test]
    fn get_exercise_returns_correct_exercise() {
        let conn = setup_test_db();
        let _ = create_library_exercise(&conn, "Squat", ExerciseType::Bodyweight)
            .expect("first exercise creation should succeed");
        let target = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = get_library_exercise(&conn, target.id())
            .expect("DB query should not fail")
            .expect("exercise should exist");

        assert_eq!(result.id(), target.id());
        assert_eq!(result.name(), target.name());
        assert_eq!(result.exercise_type(), target.exercise_type());
    }
    // --- list_library_exercises ---
    #[test]
    fn list_exercises_returns_empty_list_on_fresh_db() {
        let conn = setup_test_db();
        let result = list_library_exercises(&conn).expect("listing exercises should succeed");
        assert!(result.is_empty());
    }
    #[test]
    fn list_exercises_returns_all_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_library_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = list_library_exercises(&conn).expect("listing exercises should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.id() == exercise_1.id()));
        assert!(result.iter().any(|e| e.id() == exercise_2.id()));
    }
    #[test]
    fn list_exercises_returns_exercises_ordered_by_name() {
        let conn = setup_test_db();
        create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        create_library_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let result = list_library_exercises(&conn).expect("listing exercises should succeed");

        assert_eq!(result[0].name(), "Bench Press");
        assert_eq!(result[1].name(), "Deadlift");
        assert_eq!(result[2].name(), "Squat");
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

        crate::persistence::workouts::delete_workout(&conn, workout_id)
            .expect("delete should succeed");

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

    #[test]
    fn update_exercise_changes_name_and_type() {
        let conn = setup_test_db();
        let exercise = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let updated = update_library_exercise(
            &conn,
            exercise.id(),
            "Incline Press",
            ExerciseType::Bodyweight,
        )
        .expect("update should succeed");

        assert_eq!(updated.name(), "Incline Press");
        assert_eq!(updated.exercise_type(), ExerciseType::Bodyweight);

        let persisted = get_library_exercise(&conn, exercise.id())
            .expect("query should succeed")
            .expect("exercise should exist");
        assert_eq!(persisted.name(), "Incline Press");
        assert_eq!(persisted.exercise_type(), ExerciseType::Bodyweight);
    }

    #[test]
    fn update_exercise_keeping_its_own_name_succeeds() {
        let conn = setup_test_db();
        let exercise = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let updated =
            update_library_exercise(&conn, exercise.id(), "Squat", ExerciseType::Bodyweight)
                .expect("renaming to its own name should succeed");

        assert_eq!(updated.exercise_type(), ExerciseType::Bodyweight);
    }

    #[test]
    fn update_exercise_returns_duplicate_name_when_name_taken_by_another() {
        let conn = setup_test_db();
        create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let other = create_library_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = update_library_exercise(&conn, other.id(), "Squat", ExerciseType::Weighted);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::DuplicateName { .. })
        ));
    }

    #[test]
    fn update_exercise_returns_not_found_when_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = update_library_exercise(&conn, 9999, "Squat", ExerciseType::Weighted);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn delete_exercise_removes_it() {
        let conn = setup_test_db();
        let exercise = create_library_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        delete_library_exercise(&conn, exercise.id()).expect("delete should succeed");

        let result = get_library_exercise(&conn, exercise.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn delete_exercise_returns_not_found_when_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = delete_library_exercise(&conn, 9999);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn delete_exercise_returns_in_use_when_referenced_by_a_planned_exercise() {
        let conn = setup_test_db();
        let workout = create_test_workout(&conn);
        let exercise = create_test_exercise(&conn);
        create_planned_exercise(&conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");

        let result = delete_library_exercise(&conn, exercise.id());

        assert!(matches!(result, Err(LibraryExerciseError::InUse { .. })));
    }
}
