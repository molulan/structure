use rusqlite::{Connection, Result};

use crate::domain::planning::Mesocycle;

pub fn create_mesocycles_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mesocycles (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
        (),
    )?;
    Ok(())
}

pub fn create_mesocycle(conn: &Connection, name: &str) -> Result<Mesocycle> {
    conn.execute("INSERT INTO mesocycles (name) VALUES (?1)", (&name,))?;

    let id = conn.last_insert_rowid();

    Ok(Mesocycle::new(name, id))
}

pub fn list_mesocycles(conn: &Connection) -> rusqlite::Result<Vec<Mesocycle>> {
    let mut stmt = conn.prepare("SELECT id, name FROM mesocycles")?;
    
    let mesocycle_iter = stmt.query_map([], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        Ok(Mesocycle::new(name, id))
    })?;
    
    mesocycle_iter.collect()
}
