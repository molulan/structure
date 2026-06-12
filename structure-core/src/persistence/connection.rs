use rusqlite::Connection;

use crate::persistence::{
    library_exercises::create_library_exercises_table, mesocycles::create_mesocycles_table,
    microcycles::create_microcycles_table, planned_exercises::create_planned_exercises_table,
    sets::create_planned_sets_table, workouts::create_workouts_table,
};

fn open(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON")?;
    Ok(conn)
}

fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    create_mesocycles_table(conn)?;
    create_microcycles_table(conn)?;
    create_workouts_table(conn)?;
    create_library_exercises_table(conn)?;
    create_planned_exercises_table(conn)?;
    create_planned_sets_table(conn)?;

    Ok(())
}

pub fn init_db(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = open(db_path)?;
    create_schema(&conn)?;
    Ok(conn)
}
