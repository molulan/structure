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

pub fn list_mesocycles(conn: &Connection) -> Result<Vec<Mesocycle>> {
    let mut stmt = conn.prepare("SELECT id, name FROM mesocycles")?;

    stmt.query_map([], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        Ok(Mesocycle::new(name, id))
    })?
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::sqlite;
    
    fn setup_test_db() -> Connection {
        sqlite::init_db(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn test_list_mesocycles_empty() {
        let conn = setup_test_db();
        let result = list_mesocycles(&conn).unwrap();
        assert!(result.is_empty())
    }

    #[test]
    fn test_create_mesocycle() {
        let conn = setup_test_db();

        let name = "hypertrophy 1";
        let result = create_mesocycle(&conn, name).unwrap();

        assert_eq!(result.name(), name)
    }

    #[test]
    fn test_create_and_list_mesocycles() {
        let conn = setup_test_db();

        let mesocyle = create_mesocycle(&conn, "hypertrophy 1").unwrap();

        let result = list_mesocycles(&conn).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), mesocyle.id());
        assert_eq!(result[0].name(), mesocyle.name());
    }

    #[test]
    fn test_create_multiple_mesocycles() {
        let conn = setup_test_db();

        let mesocyle_1 = create_mesocycle(&conn, "Big Arms").unwrap();
        let mesocyle_2 = create_mesocycle(&conn, "Bigger Arms!").unwrap();
        assert_ne!(mesocyle_1.id(), mesocyle_2.id());

        let mesocycles = list_mesocycles(&conn).unwrap();
        assert_eq!(mesocycles.len(), 2);
        assert_eq!(mesocycles[0].id(), mesocyle_1.id());
        assert_eq!(mesocycles[0].name(), mesocyle_1.name());
        assert_eq!(mesocycles[1].id(), mesocyle_2.id());
        assert_eq!(mesocycles[1].name(), mesocyle_2.name());
    }
}
