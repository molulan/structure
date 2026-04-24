use crate::{
    dto::planning::MesocycleDTO,
    errors::MesocycleError,
    persistence::{mesocycles as db, sqlite},
};
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn list_mesocycles() -> Result<Vec<MesocycleDTO>, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let mesocycles = db::list_mesocycles(&conn)?;
    Ok(mesocycles.iter().map(MesocycleDTO::from).collect())
}

#[frb(sync)]
pub fn create_mesocycle(name: String) -> Result<MesocycleDTO, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;

    let mesocycle = db::create_mesocycle(&conn, &name)?;

    Ok(MesocycleDTO::from(&mesocycle))
}
