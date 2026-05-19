use flutter_rust_bridge::frb;

use crate::{
    domain::planning::ExerciseType,
    dto::planning::{ExerciseDTO, ExerciseTypeDTO, PlannedExerciseDTO},
    errors::{ExerciseError, PlannedExerciseError},
    persistence::{exercises as db, sqlite},
};

#[frb(sync)]
pub fn create_exercise(
    name: String,
    exercise_type: ExerciseTypeDTO,
) -> Result<ExerciseDTO, ExerciseError> {
    let conn = sqlite::init_db("structure.db")?;

    let exercise = db::create_exercise(&conn, &name, ExerciseType::from(exercise_type))?;

    Ok(ExerciseDTO::from(&exercise))
}

#[frb(sync)]
pub fn get_exercise(id: i64) -> Result<ExerciseDTO, ExerciseError> {
    let conn = sqlite::init_db("structure.db")?;

    let exercise = db::get_exercise(&conn, id)?.ok_or(ExerciseError::NotFound { id })?;

    Ok(ExerciseDTO::from(&exercise))
}

#[frb(sync)]
pub fn list_exercises() -> Result<Vec<ExerciseDTO>, ExerciseError> {
    let conn = sqlite::init_db("structure.db")?;

    let exercises = db::list_exercises(&conn)?;

    Ok(exercises.iter().map(ExerciseDTO::from).collect())
}

#[frb(sync)]
pub fn create_planned_exercise(
    workout_id: i64,
    exercise_id: i64,
) -> Result<PlannedExerciseDTO, PlannedExerciseError> {
    let conn = sqlite::init_db("structure.db")?;

    let planned_exercise = db::create_planned_exercise(&conn, workout_id, exercise_id)?;

    Ok(PlannedExerciseDTO::from(&planned_exercise))
}

#[frb(sync)]
pub fn get_planned_exercise(id: i64) -> Result<PlannedExerciseDTO, PlannedExerciseError> {
    let conn = sqlite::init_db("structure.db")?;

    let planned_exercise =
        db::get_planned_exercise(&conn, id)?.ok_or(PlannedExerciseError::NotFound { id })?;

    Ok(PlannedExerciseDTO::from(&planned_exercise))
}

#[frb(sync)]
pub fn list_planned_exercises(
    workout_id: i64,
) -> Result<Vec<PlannedExerciseDTO>, PlannedExerciseError> {
    let conn = sqlite::init_db("structure.db")?;

    let planned_exercises = db::list_planned_exercises(&conn, workout_id)?;

    Ok(planned_exercises
        .iter()
        .map(PlannedExerciseDTO::from)
        .collect())
}
