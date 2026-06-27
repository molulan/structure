use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::{ExerciseType, LibraryExercise, Name, NameError};

#[derive(Debug, thiserror::Error)]
pub enum LibraryExerciseError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("exercise with name '{name}' already exists")]
    DuplicateName { name: String },
    #[error("exercise {id} not found")]
    NotFound { id: i64 },
    #[error(
        "exercise {id} is referenced by one or more planned or logged exercises and cannot be deleted"
    )]
    InUse { id: i64 },
    #[error(transparent)]
    InvalidName(#[from] NameError),
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
            ),
            archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0, 1))
        )",
        (),
    )?;
    Ok(())
}

fn library_exercise_name_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM library_exercises WHERE name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub(crate) fn exercise_type_from_str(s: &str) -> ExerciseType {
    match s {
        "Bodyweight" => ExerciseType::Bodyweight,
        "WeightedBodyweight" => ExerciseType::WeightedBodyweight,
        "AssistedBodyweight" => ExerciseType::AssistedBodyweight,
        "Weighted" => ExerciseType::Weighted,
        other => panic!("Unknown exercise_type '{}'", other),
    }
}

pub fn create(
    conn: &Connection,
    name: &str,
    exercise_type: ExerciseType,
) -> Result<LibraryExercise, LibraryExerciseError> {
    let name = Name::new(name)?;

    if library_exercise_name_exists(conn, name.as_str())? {
        return Err(LibraryExerciseError::DuplicateName {
            name: name.as_str().to_string(),
        });
    }

    conn.execute(
        "INSERT INTO library_exercises (name, exercise_type) VALUES (?1, ?2)",
        params![name.as_str(), exercise_type.to_string()],
    )?;

    let id = conn.last_insert_rowid();

    Ok(LibraryExercise::new(id, name, exercise_type))
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<LibraryExercise>> {
    conn.query_row(
        "SELECT id, name, exercise_type FROM library_exercises WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let exercise_type: String = row.get(2)?;
            let exercise_type = exercise_type_from_str(exercise_type.as_str());
            let name = Name::new(name).expect("name stored in the database was validated on write");
            Ok(LibraryExercise::new(id, name, exercise_type))
        },
    )
    .optional()
}

pub fn list(conn: &Connection) -> Result<Vec<LibraryExercise>, LibraryExerciseError> {
    list_by_archived_status(conn, false)
}

pub fn list_archived(conn: &Connection) -> Result<Vec<LibraryExercise>, LibraryExerciseError> {
    list_by_archived_status(conn, true)
}

fn list_by_archived_status(
    conn: &Connection,
    archived: bool,
) -> Result<Vec<LibraryExercise>, LibraryExerciseError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, exercise_type FROM library_exercises
         WHERE archived = ?1 ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([archived], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    let mut exercises = Vec::new();
    for row in rows {
        let (id, name, exercise_type): (i64, String, String) = row?;
        let exercise_type = exercise_type_from_str(exercise_type.as_str());
        let name = Name::new(name).expect("name stored in the database was validated on write");
        exercises.push(LibraryExercise::new(id, name, exercise_type));
    }

    Ok(exercises)
}

pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    exercise_type: ExerciseType,
) -> Result<LibraryExercise, LibraryExerciseError> {
    let name = Name::new(name)?;

    if get(conn, id)?.is_none() {
        return Err(LibraryExerciseError::NotFound { id });
    }

    let name_taken: i64 = conn.query_row(
        "SELECT COUNT(*) FROM library_exercises WHERE name = ?1 AND id != ?2",
        params![name.as_str(), id],
        |row| row.get(0),
    )?;
    if name_taken > 0 {
        return Err(LibraryExerciseError::DuplicateName {
            name: name.as_str().to_string(),
        });
    }

    conn.execute(
        "UPDATE library_exercises SET name = ?1, exercise_type = ?2 WHERE id = ?3",
        params![name.as_str(), exercise_type.to_string(), id],
    )?;

    Ok(LibraryExercise::new(id, name, exercise_type))
}

pub fn update_archived_status(
    conn: &Connection,
    id: i64,
    archived: bool,
) -> Result<(), LibraryExerciseError> {
    let updated = conn.execute(
        "UPDATE library_exercises SET archived = ?1 WHERE id = ?2",
        params![archived, id],
    )?;

    if updated == 0 {
        return Err(LibraryExerciseError::NotFound { id });
    }

    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), LibraryExerciseError> {
    let in_use: i64 = conn.query_row(
        "SELECT
            (SELECT COUNT(*) FROM planned_exercises WHERE library_exercise_id = ?1)
          + (SELECT COUNT(*) FROM logged_exercises WHERE library_exercise_id = ?1)",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::MesocycleMode,
        persistence::{
            connection, logged_exercises, logged_sessions, mesocycles, microcycles,
            planned_exercises, workouts,
        },
    };

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn create_exercise_with_valid_name_and_type_succeeds() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Squat", ExerciseType::Bodyweight)
            .expect("exercise creation should succeed");

        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.exercise_type(), ExerciseType::Bodyweight);
    }
    #[test]
    fn create_exercise_with_empty_name_returns_error() {
        let conn = setup_test_db();
        let result = create(&conn, "", ExerciseType::Weighted);
        assert!(result.is_err());
    }
    #[test]
    fn create_exercise_with_duplicate_name_returns_duplicate_name_error() {
        let conn = setup_test_db();
        create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");

        let result = create(&conn, "Bench Press", ExerciseType::Weighted);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::DuplicateName { .. })
        ));
    }
    #[test]
    fn create_exercise_assigns_unique_ids_to_different_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        assert_ne!(exercise_1.id(), exercise_2.id());
    }
    #[test]
    fn all_four_exercise_types_can_be_created() {
        let conn = setup_test_db();
        let bodyweight = create(&conn, "Push Up", ExerciseType::Bodyweight)
            .expect("bodyweight exercise creation should succeed");
        let weighted_bodyweight = create(&conn, "Pull Up", ExerciseType::WeightedBodyweight)
            .expect("weighted bodyweight exercise creation should succeed");
        let assisted_bodyweight =
            create(&conn, "Assisted Pull Up", ExerciseType::AssistedBodyweight)
                .expect("assisted bodyweight exercise creation should succeed");
        let weighted = create(&conn, "Bench Press", ExerciseType::Weighted)
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
        let result = get(&conn, 9999).expect("DB query should not fail");
        assert!(result.is_none());
    }
    #[test]
    fn get_exercise_returns_correct_exercise() {
        let conn = setup_test_db();
        let _ = create(&conn, "Squat", ExerciseType::Bodyweight)
            .expect("first exercise creation should succeed");
        let target = create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = get(&conn, target.id())
            .expect("DB query should not fail")
            .expect("exercise should exist");

        assert_eq!(result.id(), target.id());
        assert_eq!(result.name(), target.name());
        assert_eq!(result.exercise_type(), target.exercise_type());
    }

    #[test]
    fn list_exercises_returns_empty_list_on_fresh_db() {
        let conn = setup_test_db();
        let result = list(&conn).expect("listing exercises should succeed");
        assert!(result.is_empty());
    }
    #[test]
    fn list_exercises_returns_all_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = list(&conn).expect("listing exercises should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.id() == exercise_1.id()));
        assert!(result.iter().any(|e| e.id() == exercise_2.id()));
    }
    #[test]
    fn list_exercises_returns_exercises_ordered_by_name() {
        let conn = setup_test_db();
        create(&conn, "Squat", ExerciseType::Weighted).expect("exercise creation should succeed");
        create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        create(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let result = list(&conn).expect("listing exercises should succeed");

        assert_eq!(result[0].name(), "Bench Press");
        assert_eq!(result[1].name(), "Deadlift");
        assert_eq!(result[2].name(), "Squat");
    }

    #[test]
    fn update_exercise_changes_name_and_type() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let updated = update(
            &conn,
            exercise.id(),
            "Incline Press",
            ExerciseType::Bodyweight,
        )
        .expect("update should succeed");

        assert_eq!(updated.name(), "Incline Press");
        assert_eq!(updated.exercise_type(), ExerciseType::Bodyweight);

        let persisted = get(&conn, exercise.id())
            .expect("query should succeed")
            .expect("exercise should exist");
        assert_eq!(persisted.name(), "Incline Press");
        assert_eq!(persisted.exercise_type(), ExerciseType::Bodyweight);
    }

    #[test]
    fn update_exercise_keeping_its_own_name_succeeds() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let updated = update(&conn, exercise.id(), "Squat", ExerciseType::Bodyweight)
            .expect("renaming to its own name should succeed");

        assert_eq!(updated.exercise_type(), ExerciseType::Bodyweight);
    }

    #[test]
    fn update_exercise_returns_duplicate_name_when_name_taken_by_another() {
        let conn = setup_test_db();
        create(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let other = create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = update(&conn, other.id(), "Squat", ExerciseType::Weighted);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::DuplicateName { .. })
        ));
    }

    #[test]
    fn update_exercise_returns_not_found_when_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = update(&conn, 9999, "Squat", ExerciseType::Weighted);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn delete_exercise_removes_it() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        delete(&conn, exercise.id()).expect("delete should succeed");

        let result = get(&conn, exercise.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn delete_exercise_returns_not_found_when_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 9999);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn delete_exercise_returns_in_use_when_referenced_by_a_planned_exercise() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");
        let microcycle =
            microcycles::create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let workout = workouts::create(&conn, microcycle.id(), "Test Workout")
            .expect("workout creation should succeed");
        let exercise = create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        planned_exercises::create(&conn, workout.id(), exercise.id())
            .expect("planned exercise creation should succeed");

        let result = delete(&conn, exercise.id());

        assert!(matches!(result, Err(LibraryExerciseError::InUse { .. })));
    }

    #[test]
    fn delete_exercise_returns_in_use_when_referenced_by_a_logged_exercise() {
        let conn = setup_test_db();
        let session = logged_sessions::create(&conn, "2026-06-26T10:00:00Z", None, None, None)
            .expect("session creation should succeed");
        let exercise = create(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        logged_exercises::create(&conn, session.id(), exercise.id(), None, None)
            .expect("logged exercise creation should succeed");

        let result = delete(&conn, exercise.id());

        assert!(matches!(result, Err(LibraryExerciseError::InUse { .. })));
    }

    #[test]
    fn new_exercises_are_not_archived() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let active = list(&conn).expect("listing should succeed");
        let archived = list_archived(&conn).expect("listing archived should succeed");

        assert!(active.iter().any(|e| e.id() == exercise.id()));
        assert!(archived.is_empty());
    }

    #[test]
    fn archiving_moves_exercise_from_active_to_archived_list() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        update_archived_status(&conn, exercise.id(), true).expect("archiving should succeed");

        let active = list(&conn).expect("listing should succeed");
        let archived = list_archived(&conn).expect("listing archived should succeed");

        assert!(!active.iter().any(|e| e.id() == exercise.id()));
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].id(), exercise.id());
    }

    #[test]
    fn unarchiving_returns_exercise_to_the_active_list() {
        let conn = setup_test_db();
        let exercise = create(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        update_archived_status(&conn, exercise.id(), true).expect("archiving should succeed");

        update_archived_status(&conn, exercise.id(), false).expect("unarchiving should succeed");

        let active = list(&conn).expect("listing should succeed");
        let archived = list_archived(&conn).expect("listing archived should succeed");

        assert!(active.iter().any(|e| e.id() == exercise.id()));
        assert!(archived.is_empty());
    }

    #[test]
    fn update_archived_status_returns_not_found_when_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = update_archived_status(&conn, 9999, true);

        assert!(matches!(
            result,
            Err(LibraryExerciseError::NotFound { id: 9999 })
        ));
    }
}
