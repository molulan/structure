use crate::dto::planning::{ExerciseTypeDTO, LibraryExerciseDTO, PlannedExerciseDTO};
use flutter_rust_bridge::frb;
use structure_core::{
    domain::planning::ExerciseType,
    persistence::{
        connection,
        exercises::{self as db, LibraryExerciseError, PlannedExerciseError},
    },
};

#[frb(sync)]
pub fn create_library_exercise(
    name: String,
    exercise_type: ExerciseTypeDTO,
) -> Result<LibraryExerciseDTO, LibraryExerciseError> {
    let conn = connection::init_db("structure.db")?;

    let exercise = db::create_library_exercise(&conn, &name, ExerciseType::from(exercise_type))?;

    Ok(LibraryExerciseDTO::from(&exercise))
}

#[frb(sync)]
pub fn get_library_exercise(id: i64) -> Result<LibraryExerciseDTO, LibraryExerciseError> {
    let conn = connection::init_db("structure.db")?;

    let exercise =
        db::get_library_exercise(&conn, id)?.ok_or(LibraryExerciseError::NotFound { id })?;

    Ok(LibraryExerciseDTO::from(&exercise))
}

#[frb(sync)]
pub fn list_library_exercises() -> Result<Vec<LibraryExerciseDTO>, LibraryExerciseError> {
    let conn = connection::init_db("structure.db")?;

    let exercises = db::list_library_exercises(&conn)?;

    Ok(exercises.iter().map(LibraryExerciseDTO::from).collect())
}

#[frb(sync)]
pub fn create_planned_exercise(
    workout_id: i64,
    library_exercise_id: i64,
) -> Result<PlannedExerciseDTO, PlannedExerciseError> {
    let conn = connection::init_db("structure.db")?;

    let planned_exercise = db::create_planned_exercise(&conn, workout_id, library_exercise_id)?;

    Ok(PlannedExerciseDTO::from(&planned_exercise))
}

#[frb(sync)]
pub fn get_planned_exercise(id: i64) -> Result<PlannedExerciseDTO, PlannedExerciseError> {
    let conn = connection::init_db("structure.db")?;

    let planned_exercise =
        db::get_planned_exercise(&conn, id)?.ok_or(PlannedExerciseError::NotFound { id })?;

    Ok(PlannedExerciseDTO::from(&planned_exercise))
}

#[frb(sync)]
pub fn list_planned_exercises(
    workout_id: i64,
) -> Result<Vec<PlannedExerciseDTO>, PlannedExerciseError> {
    let conn = connection::init_db("structure.db")?;

    let planned_exercises = db::list_planned_exercises(&conn, workout_id)?;

    Ok(planned_exercises
        .iter()
        .map(PlannedExerciseDTO::from)
        .collect())
}
