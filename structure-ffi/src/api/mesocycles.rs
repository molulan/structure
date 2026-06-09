use flutter_rust_bridge::frb;
use structure_core::{
    domain::planning::MesocycleMode,
    persistence::{
        mesocycles::{self as db, MesocycleError},
        sqlite,
    },
};
use crate::dto::planning::{MesocycleDTO, MesocycleModeDTO};


#[frb(sync)]
pub fn list_mesocycles() -> Result<Vec<MesocycleDTO>, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;
    let rows = db::list_mesocycles(&conn)?;
    Ok(rows
        .into_iter()
        .map(|r| MesocycleDTO {
            id: r.id,
            name: r.name,
            mode: MesocycleModeDTO::from(r.mode),
            microcycle_count: r.microcycle_count,
        })
        .collect())
}

#[frb(sync)]
pub fn create_mesocycle(
    name: String,
    mode: MesocycleModeDTO,
) -> Result<MesocycleDTO, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;
    let mesocycle = db::create_mesocycle(&conn, &name, MesocycleMode::from(mode))?;
    Ok(MesocycleDTO {
        id: mesocycle.id(),
        name: mesocycle.name().to_owned(),
        mode: MesocycleModeDTO::from(mesocycle.mode()),
        microcycle_count: 0,
    })
}

#[frb(sync)]
pub fn get_mesocycle(id: i64) -> Result<MesocycleDTO, MesocycleError> {
    let conn = sqlite::init_db("structure.db")?;
    let row = db::get_mesocycle(&conn, id)?.ok_or(MesocycleError::NotFound { id })?;
    Ok(MesocycleDTO {
        id: row.id,
        name: row.name,
        mode: MesocycleModeDTO::from(row.mode),
        microcycle_count: row.microcycle_count,
    })
}
