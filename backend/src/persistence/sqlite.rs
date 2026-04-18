use rusqlite::Connection;

use crate::persistence::mesocycles::create_mesocycles_table;

pub fn open_connection(db_path: &str) -> rusqlite::Result<Connection> {
    Connection::open(db_path)
}

pub fn init_db() -> rusqlite::Result<Connection> {
    let conn = open_connection("structure.db")?; //fake db path placeholder
    
    create_mesocycles_table(&conn)?;
    
    Ok(conn)
}