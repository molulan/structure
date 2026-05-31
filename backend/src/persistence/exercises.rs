use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    domain::planning::{Exercise, ExerciseType, PlannedExercise},
    errors::{ExerciseError, PlannedExerciseError},
};

pub(super) fn create_exercises_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS exercises (
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
            workout_id INTEGER NOT NULL REFERENCES workouts(id),
            exercise_id INTEGER NOT NULL REFERENCES exercises(id),
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

fn exercise_name_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM exercises WHERE name = ?1",
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

pub fn create_exercise(
    conn: &Connection,
    name: &str,
    exercise_type: ExerciseType,
) -> Result<Exercise, ExerciseError> {
    if exercise_name_exists(conn, name)? {
        return Err(ExerciseError::DuplicateName {
            name: name.to_string(),
        });
    }

    conn.execute(
        "INSERT INTO exercises (name, exercise_type) VALUES (?1, ?2)",
        params![name, exercise_type.to_string()],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Exercise::new(id, name, exercise_type))
}

pub fn get_exercise(conn: &Connection, id: i64) -> rusqlite::Result<Option<Exercise>> {
    conn.query_row(
        "SELECT id, name, exercise_type FROM exercises WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let exercise_type: String = row.get(2)?;
            let exercise_type = exercise_type_from_str(exercise_type.as_str());
            Ok(Exercise::new(id, name, exercise_type))
        },
    )
    .optional()
}

pub fn list_exercises(conn: &Connection) -> Result<Vec<Exercise>, ExerciseError> {
    let mut stmt =
        conn.prepare("SELECT id, name, exercise_type FROM exercises ORDER BY name ASC")?;

    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

    let mut exercises = Vec::new();
    for row in rows {
        let (id, name, exercise_type): (i64, String, String) = row?;
        let exercise_type = exercise_type_from_str(exercise_type.as_str());
        exercises.push(Exercise::new(id, name, exercise_type));
    }

    Ok(exercises)
}

pub fn create_planned_exercise(
    conn: &Connection,
    workout_id: i64,
    exercise_id: i64,
) -> Result<PlannedExercise, PlannedExerciseError> {
    if !workout_exists(conn, workout_id)? {
        return Err(PlannedExerciseError::AssociatedWorkoutNotFound { id: workout_id });
    }

    match get_exercise(conn, exercise_id)? {
        None => Err(PlannedExerciseError::AssociatedExerciseNotFound { id: exercise_id }),
        Some(exercise) => {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM planned_exercises WHERE workout_id = ?1",
                [workout_id],
                |row| row.get(0),
            )?;

            let position = u32::try_from(count).expect(
                "COUNT(*) is always non-negative and no workout will have 4 billion exercises",
            );

            conn.execute(
                "INSERT INTO planned_exercises (workout_id, exercise_id, position) VALUES (?1, ?2, ?3)",
                params![workout_id, exercise_id, position],
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
            "SELECT id, exercise_id, position FROM planned_exercises WHERE id = ?1",
            [id],
            |row| {
                let id: i64 = row.get(0)?;
                let exercise_id: i64 = row.get(1)?;
                let position: i64 = row.get(2)?;
                Ok((id, exercise_id, position))
            },
        )
        .optional()?;

    match row {
        None => Ok(None),
        Some((id, exercise_id, position)) => {
            let position =
                u32::try_from(position).expect("position stored in DB was originally a u32");
            let exercise = get_exercise(conn, exercise_id)?
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
        "SELECT id, exercise_id, position FROM planned_exercises WHERE workout_id = ?1 ORDER BY position ASC",
    )?;

    let rows = stmt.query_map([workout_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;

    let mut planned_exercises = Vec::new();

    for row in rows {
        let (id, exercise_id, position): (i64, i64, i64) = row?;
        let position = u32::try_from(position).expect("position stored in DB was originally a u32");
        let exercise = get_exercise(conn, exercise_id)?.expect(
            "exercise FK in planned_exercises points to nonexistent exercise — data corrupted",
        );
        let sets = super::sets::list_planned_sets(conn, id)?;
        let planned_exercise = PlannedExercise::new(id, exercise, position, sets)?;
        planned_exercises.push(planned_exercise);
    }

    Ok(planned_exercises)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::{MesocycleMode, Workout},
        persistence::{
            mesocycles::create_mesocycle, microcycles::create_microcycle, sqlite,
            workouts::create_workout,
        },
    };

    fn setup_test_db() -> Connection {
        sqlite::init_db(":memory:").expect("Failed to create test database")
    }

    fn create_test_workout(conn: &Connection) -> Workout {
        let mesocycle = create_mesocycle(conn, "Test Mesocycle", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");

        let microcycle =
            create_microcycle(conn, mesocycle.id()).expect("microcycle creation should succeed");
        create_workout(conn, microcycle.id(), "Test Workout")
            .expect("workout creation should succeed")
    }

    fn create_test_exercise(conn: &Connection) -> Exercise {
        create_exercise(conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed")
    }

    #[test]
    fn create_exercise_with_valid_name_and_type_succeeds() {
        let conn = setup_test_db();
        let exercise = create_exercise(&conn, "Squat", ExerciseType::Bodyweight)
            .expect("exercise creation should succeed");

        assert_eq!(exercise.name(), "Squat");
        assert_eq!(exercise.exercise_type(), ExerciseType::Bodyweight);
    }
    #[test]
    fn create_exercise_with_empty_name_returns_error() {
        let conn = setup_test_db();
        let result = create_exercise(&conn, "", ExerciseType::Weighted);
        assert!(result.is_err());
    }
    #[test]
    fn create_exercise_with_duplicate_name_returns_duplicate_name_error() {
        let conn = setup_test_db();
        create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");

        let result = create_exercise(&conn, "Bench Press", ExerciseType::Weighted);

        assert!(matches!(result, Err(ExerciseError::DuplicateName { .. })));
    }
    #[test]
    fn create_exercise_assigns_unique_ids_to_different_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        assert_ne!(exercise_1.id(), exercise_2.id());
    }
    #[test]
    fn all_four_exercise_types_can_be_created() {
        let conn = setup_test_db();
        let bodyweight = create_exercise(&conn, "Push Up", ExerciseType::Bodyweight)
            .expect("bodyweight exercise creation should succeed");
        let weighted_bodyweight =
            create_exercise(&conn, "Pull Up", ExerciseType::WeightedBodyweight)
                .expect("weighted bodyweight exercise creation should succeed");
        let assisted_bodyweight =
            create_exercise(&conn, "Assisted Pull Up", ExerciseType::AssistedBodyweight)
                .expect("assisted bodyweight exercise creation should succeed");
        let weighted = create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
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
        let result = get_exercise(&conn, 9999).expect("DB query should not fail");
        assert!(result.is_none());
    }
    #[test]
    fn get_exercise_returns_correct_exercise() {
        let conn = setup_test_db();
        let _ = create_exercise(&conn, "Squat", ExerciseType::Bodyweight)
            .expect("first exercise creation should succeed");
        let target = create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = get_exercise(&conn, target.id())
            .expect("DB query should not fail")
            .expect("exercise should exist");

        assert_eq!(result.id(), target.id());
        assert_eq!(result.name(), target.name());
        assert_eq!(result.exercise_type(), target.exercise_type());
    }
    // --- list_exercises ---
    #[test]
    fn list_exercises_returns_empty_list_on_fresh_db() {
        let conn = setup_test_db();
        let result = list_exercises(&conn).expect("listing exercises should succeed");
        assert!(result.is_empty());
    }
    #[test]
    fn list_exercises_returns_all_exercises() {
        let conn = setup_test_db();
        let exercise_1 = create_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");

        let result = list_exercises(&conn).expect("listing exercises should succeed");

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.id() == exercise_1.id()));
        assert!(result.iter().any(|e| e.id() == exercise_2.id()));
    }
    #[test]
    fn list_exercises_returns_exercises_ordered_by_name() {
        let conn = setup_test_db();
        create_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("exercise creation should succeed");
        create_exercise(&conn, "Deadlift", ExerciseType::Weighted)
            .expect("exercise creation should succeed");

        let result = list_exercises(&conn).expect("listing exercises should succeed");

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
        let exercise_1 = create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("second exercise creation should succeed");
        let exercise_3 = create_exercise(&conn, "Deadlift", ExerciseType::Weighted)
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
        let exercise_1 = create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_exercise(&conn, "Squat", ExerciseType::Weighted)
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
        let exercise_1 = create_exercise(&conn, "Squat", ExerciseType::Weighted)
            .expect("first exercise creation should succeed");
        let exercise_2 = create_exercise(&conn, "Bench Press", ExerciseType::Weighted)
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
            create_exercise(&conn, "Arnolds Favorite Armblaster", ExerciseType::Weighted)
                .expect("exercise creation should succeed");

        let exercise_2 = create_exercise(
            &conn,
            "Arnolds Second Favorite Armblaster",
            ExerciseType::Bodyweight,
        )
        .expect("exercise creation should succeed");

        let exercise_3 = create_exercise(&conn, "squat", ExerciseType::Weighted)
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
}
