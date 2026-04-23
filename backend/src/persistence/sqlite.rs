use rusqlite::Connection;

use crate::persistence::{
    mesocycles::create_mesocycles_table, microcycles::create_microcycles_table,
};

pub fn open_connection(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON")?;
    Ok(conn)
}

pub fn init_db(db_path: &str) -> rusqlite::Result<Connection> {
    let conn = open_connection(db_path)?;

    create_mesocycles_table(&conn)?;
    create_microcycles_table(&conn)?;

    Ok(conn)
}
