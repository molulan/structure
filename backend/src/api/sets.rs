use flutter_rust_bridge::frb;

use crate::{
    dto::planning::{LoadDTO, SetDTO, SetTypeDTO},
    persistence::{
        sets::{self as db, SetError},
        sqlite,
    },
};

#[frb(sync)]
pub fn create_planned_set(
    planned_exercise_id: i64,
    load: LoadDTO,
    reps: Option<u32>,
    set_type: SetTypeDTO,
) -> Result<SetDTO, SetError> {
    let conn = sqlite::init_db("structure.db")?;

    let set = db::create_planned_set(
        &conn,
        planned_exercise_id,
        load.into(),
        reps,
        set_type.into(),
    )?;

    Ok(SetDTO::from(set))
}

#[frb(sync)]
pub fn list_planned_sets(planned_exercise_id: i64) -> Result<Vec<SetDTO>, SetError> {
    let conn = sqlite::init_db("structure.db")?;

    let sets = db::list_planned_sets(&conn, planned_exercise_id)?;

    Ok(sets.into_iter().map(SetDTO::from).collect())
}
