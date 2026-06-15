use crate::dto::planning::WorkoutDTO;
use flutter_rust_bridge::frb;
use structure_core::persistence::{
    connection,
    workouts::{self as db, WorkoutError},
};

#[frb(sync)]
pub fn list_workouts(microcycle_id: i64) -> Result<Vec<WorkoutDTO>, WorkoutError> {
    let conn = connection::init_db("structure.db")?;

    let workouts = db::list(&conn, microcycle_id)?;

    Ok(workouts.iter().map(WorkoutDTO::from).collect())
}

#[frb(sync)]
pub fn create_workout(microcycle_id: i64, name: String) -> Result<WorkoutDTO, WorkoutError> {
    let conn = connection::init_db("structure.db")?;

    let workout = db::create(&conn, microcycle_id, &name)?;

    Ok(WorkoutDTO::from(&workout))
}

#[frb(sync)]
pub fn get_workout(id: i64) -> Result<WorkoutDTO, WorkoutError> {
    let conn = connection::init_db("structure.db")?;

    let workout = db::get(&conn, id)?.ok_or(WorkoutError::NotFound { id })?;

    Ok(WorkoutDTO::from(&workout))
}
