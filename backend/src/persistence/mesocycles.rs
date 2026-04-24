use rusqlite::{Connection, OptionalExtension, Result};

use crate::domain::planning::Mesocycle;

pub fn create_mesocycles_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mesocycles (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL CHECK(length(name) > 0)
        )",
        (),
    )?;
    Ok(())
}

pub fn create_mesocycle(conn: &Connection, name: &str) -> Result<Mesocycle> {
    conn.execute("INSERT INTO mesocycles (name) VALUES (?1)", (&name,))?;

    let id = conn.last_insert_rowid();

    Ok(Mesocycle::new(id, name))
}

pub fn get_mesocycle(conn: &Connection, id: i64) -> Result<Option<Mesocycle>> {
    conn.query_row(
        "SELECT id, name FROM mesocycles WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            Ok(Mesocycle::new(id, name))
        },
    )
    .optional()
}

pub fn list_mesocycles(conn: &Connection) -> Result<Vec<Mesocycle>> {
    let mut stmt = conn.prepare("SELECT id, name FROM mesocycles ORDER BY id ASC")?;

    stmt.query_map([], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        Ok(Mesocycle::new(id, name))
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
    fn get_mesocycle_returns_none_on_invalid_id() {
        let conn = setup_test_db();

        let result = get_mesocycle(&conn, 1234).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn get_mesocycle_returns_correct_mesocycle() {
        let conn = setup_test_db();
        let _ = create_mesocycle(&conn, "small arms").unwrap();
        let target = create_mesocycle(&conn, "BIG ARMS").unwrap();

        let result = get_mesocycle(&conn, target.id())
            .expect("DB query should not fail")
            .expect("mesocycle should exist");

        assert_eq!(target, result)
    }

    #[test]
    fn list_mesocycles_returns_empty_list_on_fresh_db() {
        let conn = setup_test_db();
        let result = list_mesocycles(&conn).unwrap();
        assert!(result.is_empty())
    }

    #[test]
    fn create_mesocycle_generates_mesocycle_with_correct_name() {
        let conn = setup_test_db();

        let name = "hypertrophy 1";
        let result = create_mesocycle(&conn, name).unwrap();

        assert_eq!(result.name(), name)
    }

    #[test]
    fn created_mesocycle_appears_in_list_with_correct_id_and_name() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "hypertrophy 1").unwrap();

        let result = list_mesocycles(&conn).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), mesocycle.id());
        assert_eq!(result[0].name(), mesocycle.name());
    }

    #[test]
    fn multiple_mesocycles_get_unique_ids_and_appear_in_list() {
        let conn = setup_test_db();

        let mesocycle_1 = create_mesocycle(&conn, "Big Arms").unwrap();
        let mesocycle_2 = create_mesocycle(&conn, "Bigger Arms!").unwrap();
        assert_ne!(mesocycle_1.id(), mesocycle_2.id());

        let mesocycles = list_mesocycles(&conn).unwrap();
        assert_eq!(mesocycles.len(), 2);
        assert_eq!(mesocycles[0].id(), mesocycle_1.id());
        assert_eq!(mesocycles[0].name(), mesocycle_1.name());
        assert_eq!(mesocycles[1].id(), mesocycle_2.id());
        assert_eq!(mesocycles[1].name(), mesocycle_2.name());
    }
}
