use flutter_rust_bridge::frb;

use crate::{
    dto::planning::MicrocycleDTO,
    errors::MicrocycleError,
    persistence::{microcycles as db, sqlite},
};

#[frb(sync)]
pub fn list_microcycles(mesocycle_id: i64) -> Result<Vec<MicrocycleDTO>, MicrocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let microcycles = db::list_microcycles(&conn, mesocycle_id)?;

    Ok(microcycles.iter().map(MicrocycleDTO::from).collect())
}

#[frb(sync)]
pub fn create_microcycle(mesocycle_id: i64) -> Result<MicrocycleDTO, MicrocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let microcycle = db::create_microcycle(&conn, mesocycle_id)?;

    Ok(MicrocycleDTO::from(&microcycle))
}

#[frb(sync)]
pub fn get_microcycle(id: i64) -> Result<MicrocycleDTO, MicrocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let microcycle = db::get_microcycle(&conn, id)?.ok_or(MicrocycleError::NotFound { id })?;

    Ok(MicrocycleDTO::from(&microcycle))
}
