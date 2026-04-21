use rusqlite::Connection;

use crate::persistence::{mesocycles::create_mesocycles_table, microcycles::create_microcycle_table};

pub fn open_connection(db_path: &str) -> rusqlite::Result<Connection> {
    Connection::open(db_path)
}

pub fn init_db(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = open_connection(db_path)?;

    create_mesocycles_table(&conn)?;
    create_microcycle_table(&conn)?;

    Ok(conn)
}
