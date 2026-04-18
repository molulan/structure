use crate::{
    dto::planning::MesocycleDTO,
    persistence::{mesocycles as db, sqlite::init_db},
};
use flutter_rust_bridge::frb;

#[frb(sync)]
pub fn list_mesocycles() -> Result<Vec<MesocycleDTO>, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    let mesocycles = db::list_mesocycles(&conn).map_err(|e| e.to_string())?;
    
    Ok(
        mesocycles.iter()
            .map(|mesocycle| MesocycleDTO::from(mesocycle))
            .collect()
    )
}

#[frb(sync)]
pub fn create_mesocycle(name: String) -> Result<MesocycleDTO, String> {
    let conn = init_db().map_err(|e| e.to_string())?;
    
    let mesocycle = db::create_mesocycle(&conn, &name).map_err(|e| e.to_string())?;
    
    Ok(MesocycleDTO::from(&mesocycle))
}