use crate::{
    domain::planning::MesocycleMode, dto::planning::{MesocycleDTO, MesocycleModeDTO}, errors::MesocycleError, persistence::{mesocycles as db, sqlite}
};
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn list_mesocycles() -> Result<Vec<MesocycleDTO>, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let mesocycles = db::list_mesocycles(&conn)?;
    Ok(mesocycles.iter().map(MesocycleDTO::from).collect())
}

#[frb(sync)]
pub fn create_mesocycle(name: String, mode: MesocycleModeDTO) -> Result<MesocycleDTO, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let mesocycle = db::create_mesocycle(&conn, &name, MesocycleMode::from(mode))?;

    Ok(MesocycleDTO::from(&mesocycle))
}

#[frb(sync)]
pub fn get_mesocycle(id: i64) -> Result<MesocycleDTO, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let mesocycle = db::get_mesocycle(&conn, id)?.ok_or(MesocycleError::NotFound { id })?;

    Ok(MesocycleDTO::from(&mesocycle))
}
