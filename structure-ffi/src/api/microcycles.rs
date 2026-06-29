use crate::dto::planning::{MicrocycleDTO, PhaseDTO};
use flutter_rust_bridge::frb;
use structure_core::persistence::{
    connection,
    microcycles::{self as db, MicrocycleError},
};

#[frb(sync)]
pub fn list_microcycles(mesocycle_id: i64) -> Result<Vec<MicrocycleDTO>, MicrocycleError> {
    let conn = connection::init_db("structure.db")?;

    let microcycles = db::list(&conn, mesocycle_id)?;

    Ok(microcycles.iter().map(MicrocycleDTO::from).collect())
}

#[frb(sync)]
pub fn create_microcycle(mesocycle_id: i64) -> Result<MicrocycleDTO, MicrocycleError> {
    let conn = connection::init_db("structure.db")?;

    let microcycle = db::create(&conn, mesocycle_id)?;

    Ok(MicrocycleDTO::from(&microcycle))
}

#[frb(sync)]
pub fn get_microcycle(id: i64) -> Result<MicrocycleDTO, MicrocycleError> {
    let conn = connection::init_db("structure.db")?;

    let microcycle = db::get(&conn, id)?.ok_or(MicrocycleError::NotFound { id })?;

    Ok(MicrocycleDTO::from(&microcycle))
}

#[frb(sync)]
pub fn set_microcycle_phase(id: i64, phase: Option<PhaseDTO>) -> Result<(), MicrocycleError> {
    let conn = connection::init_db("structure.db")?;

    db::set_phase(&conn, id, phase.map(Into::into))
}
