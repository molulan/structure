use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::planning::{ExerciseType, LibraryExercise, MuscleGroup, Name, NameError};

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
    #[error("muscle {muscle} is listed as both the primary and a secondary")]
    SecondaryMatchesPrimary { muscle: &'static str },
    #[error("corrupt library exercise data in the database: {0}")]
    Corrupt(String),
}

fn corrupt(detail: impl std::fmt::Display) -> LibraryExerciseError {
    LibraryExerciseError::Corrupt(detail.to_string())
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
            primary_muscle_group TEXT NOT NULL CHECK(
                primary_muscle_group IN (
                    'Chest', 'Back', 'Traps', 'Shoulders', 'Quads', 'Hamstrings',
                    'Glutes', 'Biceps', 'Triceps', 'Calves'
                )
            ),
            archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0, 1))
        )",
        (),
    )?;
    // Secondary muscles are a set per exercise, so they live in their own table.
    // The composite primary key rejects a muscle listed twice for one exercise.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS library_exercise_secondary_muscles (
            library_exercise_id INTEGER NOT NULL
                REFERENCES library_exercises(id) ON DELETE CASCADE,
            muscle_group TEXT NOT NULL CHECK(
                muscle_group IN (
                    'Chest', 'Back', 'Traps', 'Shoulders', 'Quads', 'Hamstrings',
                    'Glutes', 'Biceps', 'Triceps', 'Calves'
                )
            ),
            PRIMARY KEY (library_exercise_id, muscle_group)
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

fn muscle_group_from_str(s: &str) -> Result<MuscleGroup, LibraryExerciseError> {
    match s {
        "Chest" => Ok(MuscleGroup::Chest),
        "Back" => Ok(MuscleGroup::Back),
        "Traps" => Ok(MuscleGroup::Traps),
        "Shoulders" => Ok(MuscleGroup::Shoulders),
        "Quads" => Ok(MuscleGroup::Quads),
        "Hamstrings" => Ok(MuscleGroup::Hamstrings),
        "Glutes" => Ok(MuscleGroup::Glutes),
        "Biceps" => Ok(MuscleGroup::Biceps),
        "Triceps" => Ok(MuscleGroup::Triceps),
        "Calves" => Ok(MuscleGroup::Calves),
        other => Err(corrupt(format!("unknown muscle_group: {other}"))),
    }
}

/// Reads an exercise's secondary muscles, surfacing an out-of-set value as a
/// typed [`LibraryExerciseError::Corrupt`] rather than panicking.
fn secondary_muscle_groups(
    conn: &Connection,
    exercise_id: i64,
) -> Result<Vec<MuscleGroup>, LibraryExerciseError> {
    let mut stmt = conn.prepare(
        "SELECT muscle_group FROM library_exercise_secondary_muscles
         WHERE library_exercise_id = ?1 ORDER BY muscle_group ASC",
    )?;
    let rows = stmt.query_map([exercise_id], |row| row.get::<_, String>(0))?;

    let mut muscles = Vec::new();
    for row in rows {
        muscles.push(muscle_group_from_str(&row?)?);
    }
    Ok(muscles)
}

fn insert_secondary_muscle_groups(
    conn: &Connection,
    exercise_id: i64,
    secondaries: &[MuscleGroup],
) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO library_exercise_secondary_muscles (library_exercise_id, muscle_group)
         VALUES (?1, ?2)",
    )?;
    for muscle in secondaries {
        stmt.execute(params![exercise_id, muscle.as_str()])?;
    }
    Ok(())
}

/// Rejects a secondary that duplicates the primary (which would double-count the
/// muscle's volume) and drops repeats. The result is sorted by name so a written
/// exercise matches the order [`secondary_muscle_groups`] reads back.
fn checked_secondaries(
    primary: MuscleGroup,
    secondaries: &[MuscleGroup],
) -> Result<Vec<MuscleGroup>, LibraryExerciseError> {
    let mut checked = Vec::new();
    for &muscle in secondaries {
        if muscle == primary {
            return Err(LibraryExerciseError::SecondaryMatchesPrimary {
                muscle: muscle.as_str(),
            });
        }
        if !checked.contains(&muscle) {
            checked.push(muscle);
        }
    }
    checked.sort_by_key(|muscle| muscle.as_str());
    Ok(checked)
}

/// Builds a [`LibraryExercise`] from its persisted columns, surfacing an
/// out-of-set muscle group as a typed [`LibraryExerciseError::Corrupt`] rather
/// than panicking.
fn decode_exercise(
    id: i64,
    name: String,
    exercise_type: String,
    primary_muscle_group: String,
    secondary_muscle_groups: Vec<MuscleGroup>,
) -> Result<LibraryExercise, LibraryExerciseError> {
    let exercise_type = exercise_type_from_str(&exercise_type);
    let primary_muscle_group = muscle_group_from_str(&primary_muscle_group)?;
    let name = Name::new(name).map_err(corrupt)?;
    Ok(LibraryExercise::new(
        id,
        name,
        exercise_type,
        primary_muscle_group,
        secondary_muscle_groups,
    ))
}

pub fn create(
    conn: &Connection,
    name: &str,
    exercise_type: ExerciseType,
    primary_muscle_group: MuscleGroup,
    secondary_muscle_groups: &[MuscleGroup],
) -> Result<LibraryExercise, LibraryExerciseError> {
    let name = Name::new(name)?;
    let secondary_muscle_groups =
        checked_secondaries(primary_muscle_group, secondary_muscle_groups)?;

    if library_exercise_name_exists(conn, name.as_str())? {
        return Err(LibraryExerciseError::DuplicateName {
            name: name.as_str().to_string(),
        });
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO library_exercises (name, exercise_type, primary_muscle_group)
         VALUES (?1, ?2, ?3)",
        params![
            name.as_str(),
            exercise_type.as_str(),
            primary_muscle_group.as_str()
        ],
    )?;
    let id = tx.last_insert_rowid();
    insert_secondary_muscle_groups(&tx, id, &secondary_muscle_groups)?;
    tx.commit()?;

    Ok(LibraryExercise::new(
        id,
        name,
        exercise_type,
        primary_muscle_group,
        secondary_muscle_groups,
    ))
}

pub fn get(conn: &Connection, id: i64) -> Result<Option<LibraryExercise>, LibraryExerciseError> {
    let row = conn
        .query_row(
            "SELECT id, name, exercise_type, primary_muscle_group
             FROM library_exercises WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?;

    let Some((id, name, exercise_type, primary_muscle_group)) = row else {
        return Ok(None);
    };
    let secondary_muscle_groups = secondary_muscle_groups(conn, id)?;
    Ok(Some(decode_exercise(
        id,
        name,
        exercise_type,
        primary_muscle_group,
        secondary_muscle_groups,
    )?))
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
        "SELECT id, name, exercise_type, primary_muscle_group FROM library_exercises
         WHERE archived = ?1 ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([archived], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    // Collect the base rows before fetching each exercise's secondary muscles,
    // so the secondary lookups don't borrow the in-flight statement.
    let base: Vec<(i64, String, String, String)> = rows.collect::<rusqlite::Result<_>>()?;

    let mut exercises = Vec::new();
    for (id, name, exercise_type, primary_muscle_group) in base {
        let secondary_muscle_groups = secondary_muscle_groups(conn, id)?;
        exercises.push(decode_exercise(
            id,
            name,
            exercise_type,
            primary_muscle_group,
            secondary_muscle_groups,
        )?);
    }

    Ok(exercises)
}

pub fn update(
    conn: &Connection,
    id: i64,
    name: &str,
    exercise_type: ExerciseType,
    primary_muscle_group: MuscleGroup,
    secondary_muscle_groups: &[MuscleGroup],
) -> Result<LibraryExercise, LibraryExerciseError> {
    let name = Name::new(name)?;
    let secondary_muscle_groups =
        checked_secondaries(primary_muscle_group, secondary_muscle_groups)?;

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

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE library_exercises SET name = ?1, exercise_type = ?2, primary_muscle_group = ?3
         WHERE id = ?4",
        params![
            name.as_str(),
            exercise_type.as_str(),
            primary_muscle_group.as_str(),
            id
        ],
    )?;
    tx.execute(
        "DELETE FROM library_exercise_secondary_muscles WHERE library_exercise_id = ?1",
        [id],
    )?;
    insert_secondary_muscle_groups(&tx, id, &secondary_muscle_groups)?;
    tx.commit()?;

    Ok(LibraryExercise::new(
        id,
        name,
        exercise_type,
        primary_muscle_group,
        secondary_muscle_groups,
    ))
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
        let exercise = create(
            &conn,
            "Squat",
            ExerciseType::Bodyweight,
            MuscleGroup::Quads,
            &[],
        )
        .expect("exercise creation should succeed");

        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.exercise_type(), ExerciseType::Bodyweight);
    }
    #[test]
    fn create_exercise_with_empty_name_returns_error() {
        let conn = setup_test_db();
        let result = create(&conn, "", ExerciseType::Weighted, MuscleGroup::Chest, &[]);
        assert!(result.is_err());
    }
    #[test]
    fn create_exercise_with_duplicate_name_returns_duplicate_name_error() {
        let conn = setup_test_db();
        create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
        .expect("first exercise creation should succeed");

        let result = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        );

        assert!(matches!(
            result,
            Err(LibraryExerciseError::DuplicateName { .. })
        ));
    }
    #[test]
    fn create_exercise_assigns_unique_ids_to_different_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("first exercise creation should succeed");
        let exercise_2 = create(
            &conn,
            "Deadlift",
            ExerciseType::Weighted,
            MuscleGroup::Back,
            &[],
        )
        .expect("second exercise creation should succeed");

        assert_ne!(exercise_1.id(), exercise_2.id());
    }
    #[test]
    fn all_four_exercise_types_can_be_created() {
        let conn = setup_test_db();
        let bodyweight = create(
            &conn,
            "Push Up",
            ExerciseType::Bodyweight,
            MuscleGroup::Chest,
            &[],
        )
        .expect("bodyweight exercise creation should succeed");
        let weighted_bodyweight = create(
            &conn,
            "Pull Up",
            ExerciseType::WeightedBodyweight,
            MuscleGroup::Back,
            &[],
        )
        .expect("weighted bodyweight exercise creation should succeed");
        let assisted_bodyweight = create(
            &conn,
            "Assisted Pull Up",
            ExerciseType::AssistedBodyweight,
            MuscleGroup::Back,
            &[],
        )
        .expect("assisted bodyweight exercise creation should succeed");
        let weighted = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
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
        let _ = create(
            &conn,
            "Squat",
            ExerciseType::Bodyweight,
            MuscleGroup::Quads,
            &[],
        )
        .expect("first exercise creation should succeed");
        let target = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
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
        let exercise_1 = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("first exercise creation should succeed");
        let exercise_2 = create(
            &conn,
            "Deadlift",
            ExerciseType::Weighted,
            MuscleGroup::Back,
            &[],
        )
        .expect("second exercise creation should succeed");

        let result = list(&conn).expect("listing exercises should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.id() == exercise_1.id()));
        assert!(result.iter().any(|e| e.id() == exercise_2.id()));
    }
    #[test]
    fn list_exercises_returns_exercises_ordered_by_name() {
        let conn = setup_test_db();
        create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("exercise creation should succeed");
        create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
        .expect("exercise creation should succeed");
        create(
            &conn,
            "Deadlift",
            ExerciseType::Weighted,
            MuscleGroup::Back,
            &[],
        )
        .expect("exercise creation should succeed");

        let result = list(&conn).expect("listing exercises should succeed");

        assert_eq!(result[0].name(), "Bench Press");
        assert_eq!(result[1].name(), "Deadlift");
        assert_eq!(result[2].name(), "Squat");
    }

    #[test]
    fn update_exercise_changes_name_and_type() {
        let conn = setup_test_db();
        let exercise = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
        .expect("exercise creation should succeed");

        let updated = update(
            &conn,
            exercise.id(),
            "Incline Press",
            ExerciseType::Bodyweight,
            MuscleGroup::Chest,
            &[],
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
        let exercise = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("exercise creation should succeed");

        let updated = update(
            &conn,
            exercise.id(),
            "Squat",
            ExerciseType::Bodyweight,
            MuscleGroup::Quads,
            &[],
        )
        .expect("renaming to its own name should succeed");

        assert_eq!(updated.exercise_type(), ExerciseType::Bodyweight);
    }

    #[test]
    fn update_exercise_returns_duplicate_name_when_name_taken_by_another() {
        let conn = setup_test_db();
        create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("first exercise creation should succeed");
        let other = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
        .expect("second exercise creation should succeed");

        let result = update(
            &conn,
            other.id(),
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        );

        assert!(matches!(
            result,
            Err(LibraryExerciseError::DuplicateName { .. })
        ));
    }

    #[test]
    fn update_exercise_returns_not_found_when_exercise_does_not_exist() {
        let conn = setup_test_db();

        let result = update(
            &conn,
            9999,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        );

        assert!(matches!(
            result,
            Err(LibraryExerciseError::NotFound { id: 9999 })
        ));
    }

    #[test]
    fn delete_exercise_removes_it() {
        let conn = setup_test_db();
        let exercise = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
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
        let exercise = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
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
        let exercise = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[],
        )
        .expect("exercise creation should succeed");
        logged_exercises::create(&conn, session.id(), exercise.id(), None, None)
            .expect("logged exercise creation should succeed");

        let result = delete(&conn, exercise.id());

        assert!(matches!(result, Err(LibraryExerciseError::InUse { .. })));
    }

    #[test]
    fn new_exercises_are_not_archived() {
        let conn = setup_test_db();
        let exercise = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("exercise creation should succeed");

        let active = list(&conn).expect("listing should succeed");
        let archived = list_archived(&conn).expect("listing archived should succeed");

        assert!(active.iter().any(|e| e.id() == exercise.id()));
        assert!(archived.is_empty());
    }

    #[test]
    fn archiving_moves_exercise_from_active_to_archived_list() {
        let conn = setup_test_db();
        let exercise = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
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
        let exercise = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
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

    #[test]
    fn create_persists_primary_and_secondary_muscle_groups_and_reads_them_back() {
        let conn = setup_test_db();
        let created = create(
            &conn,
            "Bench Press",
            ExerciseType::Weighted,
            MuscleGroup::Chest,
            &[MuscleGroup::Triceps, MuscleGroup::Shoulders],
        )
        .expect("exercise creation should succeed");

        // Secondaries are stored and returned sorted by name, so the written and
        // read-back orders agree regardless of the order they were passed in.
        assert_eq!(created.primary_muscle_group(), MuscleGroup::Chest);
        assert_eq!(
            created.secondary_muscle_groups(),
            [MuscleGroup::Shoulders, MuscleGroup::Triceps]
        );

        let fetched = get(&conn, created.id())
            .expect("query should succeed")
            .expect("exercise should exist");
        assert_eq!(fetched.primary_muscle_group(), MuscleGroup::Chest);
        assert_eq!(
            fetched.secondary_muscle_groups(),
            [MuscleGroup::Shoulders, MuscleGroup::Triceps]
        );
    }

    #[test]
    fn create_defaults_to_no_secondary_muscle_groups() {
        let conn = setup_test_db();
        let created = create(
            &conn,
            "Squat",
            ExerciseType::Weighted,
            MuscleGroup::Quads,
            &[],
        )
        .expect("exercise creation should succeed");

        assert!(created.secondary_muscle_groups().is_empty());
    }

    #[test]
    fn every_muscle_group_round_trips_as_a_primary() {
        let conn = setup_test_db();
        let muscles = [
            MuscleGroup::Chest,
            MuscleGroup::Back,
            MuscleGroup::Traps,
            MuscleGroup::Shoulders,
            MuscleGroup::Quads,
            MuscleGroup::Hamstrings,
            MuscleGroup::Glutes,
            MuscleGroup::Biceps,
            MuscleGroup::Triceps,
            MuscleGroup::Calves,
        ];

        for (index, muscle) in muscles.into_iter().enumerate() {
            let created = create(
                &conn,
                &format!("Exercise {index}"),
                ExerciseType::Weighted,
                muscle,
                &[],
            )
            .expect("exercise creation should succeed");

            let fetched = get(&conn, created.id())
                .expect("query should succeed")
                .expect("exercise should exist");
            assert_eq!(fetched.primary_muscle_group(), muscle);
        }
    }

    #[test]
    fn create_rejects_a_secondary_that_matches_the_primary() {
        let conn = setup_test_db();

        let result = create(
            &conn,
            "Curl",
            ExerciseType::Weighted,
            MuscleGroup::Biceps,
            &[MuscleGroup::Biceps],
        );

        assert!(matches!(
            result,
            Err(LibraryExerciseError::SecondaryMatchesPrimary { .. })
        ));
    }

    #[test]
    fn create_drops_repeated_secondary_muscle_groups() {
        let conn = setup_test_db();
        let created = create(
            &conn,
            "Deadlift",
            ExerciseType::Weighted,
            MuscleGroup::Back,
            &[MuscleGroup::Hamstrings, MuscleGroup::Hamstrings],
        )
        .expect("exercise creation should succeed");

        assert_eq!(created.secondary_muscle_groups(), [MuscleGroup::Hamstrings]);
    }

    #[test]
    fn update_replaces_primary_and_secondary_muscle_groups() {
        let conn = setup_test_db();
        let exercise = create(
            &conn,
            "Row",
            ExerciseType::Weighted,
            MuscleGroup::Back,
            &[MuscleGroup::Biceps],
        )
        .expect("exercise creation should succeed");

        let updated = update(
            &conn,
            exercise.id(),
            "Row",
            ExerciseType::Weighted,
            MuscleGroup::Back,
            &[MuscleGroup::Traps],
        )
        .expect("update should succeed");

        assert_eq!(updated.secondary_muscle_groups(), [MuscleGroup::Traps]);

        let fetched = get(&conn, exercise.id())
            .expect("query should succeed")
            .expect("exercise should exist");
        assert_eq!(fetched.secondary_muscle_groups(), [MuscleGroup::Traps]);
    }

    #[test]
    fn deleting_an_exercise_cascades_to_its_secondary_muscle_groups() {
        let conn = setup_test_db();
        let exercise = create(
            &conn,
            "Pull Up",
            ExerciseType::WeightedBodyweight,
            MuscleGroup::Back,
            &[MuscleGroup::Biceps],
        )
        .expect("exercise creation should succeed");

        delete(&conn, exercise.id()).expect("delete should succeed");

        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM library_exercise_secondary_muscles
                 WHERE library_exercise_id = ?1",
                [exercise.id()],
                |row| row.get(0),
            )
            .expect("count query should succeed");
        assert_eq!(remaining, 0);
    }

    #[test]
    fn schema_rejects_an_out_of_set_primary_muscle_group() {
        let conn = setup_test_db();

        let result = conn.execute(
            "INSERT INTO library_exercises (name, exercise_type, primary_muscle_group)
             VALUES ('Curl', 'Weighted', 'Forearms')",
            [],
        );

        assert!(
            result.is_err(),
            "an unknown primary_muscle_group must be rejected"
        );
    }

    #[test]
    fn read_surfaces_a_corrupt_name_as_a_typed_error_rather_than_panicking() {
        let conn = setup_test_db();
        // A single space passes the length > 0 CHECK but is not a valid Name, so
        // decoding it must degrade to a typed error instead of panicking.
        conn.execute(
            "INSERT INTO library_exercises (name, exercise_type, primary_muscle_group)
             VALUES (' ', 'Weighted', 'Chest')",
            [],
        )
        .expect("the raw row itself satisfies the CHECK constraints");

        let result = list(&conn);

        assert!(matches!(result, Err(LibraryExerciseError::Corrupt(_))));
    }
}
