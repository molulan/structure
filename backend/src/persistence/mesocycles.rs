use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::domain::planning::{Mesocycle, MesocycleMode};

pub(super) fn create_mesocycles_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mesocycles (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL CHECK(length(name) > 0),
            mode TEXT NOT NULL CHECK(
                mode IN ('Algorithmic', 'Manual')
            )
        )",
        (),
    )?;
    Ok(())
}

pub fn create_mesocycle(conn: &Connection, name: &str, mode: MesocycleMode) -> Result<Mesocycle> {
    conn.execute(
        "INSERT INTO mesocycles (name, mode) VALUES (?1, ?2)",
        params![name, mode.to_string()]
    )?;

    let id = conn.last_insert_rowid();

    Ok(Mesocycle::new(id, name, mode))
}

pub fn get_mesocycle(conn: &Connection, id: i64) -> Result<Option<Mesocycle>> {
    conn.query_row(
        "SELECT id, name, mode FROM mesocycles WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let mode: String = row.get(2)?;
            let mode = mesocycle_mode_from_str(&mode);
            Ok(Mesocycle::new(id, name, mode))
        },
    )
    .optional()
}

pub fn list_mesocycles(conn: &Connection) -> Result<Vec<Mesocycle>> {
    let mut stmt = conn.prepare("SELECT id, name, mode FROM mesocycles ORDER BY id ASC")?;

    stmt.query_map([], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        let mode: String = row.get(2)?;
        let mode = mesocycle_mode_from_str(&mode);
        Ok(Mesocycle::new(id, name, mode))
    })?
    .collect()
}

fn mesocycle_mode_from_str(s: &str) -> MesocycleMode {
    match s {
        "Algorithmic" => MesocycleMode::Algorithmic,
        "Manual" => MesocycleMode::Manual,
        other => panic!("Unexpected mesocyle mode: {}", other)
    }
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

        let result = get_mesocycle(&conn, 1234).expect("should return None on invalid id");

        assert!(result.is_none());
    }

    #[test]
    fn get_mesocycle_returns_correct_mesocycle() {
        let conn = setup_test_db();
        let _ = create_mesocycle(&conn, "small arms", MesocycleMode::Algorithmic)
            .expect("mesocycle creation should succeed");
        let target =
            create_mesocycle(&conn, "BIG ARMS", MesocycleMode::Manual)
                .expect("mesocycle creation should succeed");

        let result = get_mesocycle(&conn, target.id())
            .expect("DB query should not fail")
            .expect("mesocycle should exist");

        assert_eq!(target, result)
    }

    #[test]
    fn list_mesocycles_returns_empty_list_on_fresh_db() {
        let conn = setup_test_db();
        let result =
            list_mesocycles(&conn).expect("listing mesoocycles for a valid id should succeed");
        assert!(result.is_empty())
    }

    #[test]
    fn create_mesocycle_generates_mesocycle_with_correct_name() {
        let conn = setup_test_db();

        let name = "hypertrophy 1";
        let result = create_mesocycle(&conn, name, MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        assert_eq!(result.name(), name)
    }

    #[test]
    fn created_mesocycle_appears_in_list_with_correct_id_name_and_mode() {
        let conn = setup_test_db();

        let mesocycle =
            create_mesocycle(&conn, "hypertrophy 1", MesocycleMode::Manual)
                .expect("mesocycle creation should succeed");

        let result =
            list_mesocycles(&conn).expect("listing mesocycles for a valid id should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), mesocycle.id());
        assert_eq!(result[0].name(), mesocycle.name());
        assert_eq!(result[0].mode(), mesocycle.mode());
    }

    #[test]
    fn multiple_mesocycles_get_unique_ids_and_appear_in_list() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle_1 =
            create_mesocycle(&conn, "Big Arms", mode).expect("mesocycle creation should succeed");
        let mesocycle_2 =
            create_mesocycle(&conn, "Bigger Arms!", mode).expect("mesocycle creation should succeed");
        assert_ne!(mesocycle_1.id(), mesocycle_2.id());

        let mesocycles =
            list_mesocycles(&conn).expect("listing mesocycles for a valid id should succeed");
        assert_eq!(mesocycles.len(), 2);
        assert_eq!(mesocycles[0].id(), mesocycle_1.id());
        assert_eq!(mesocycles[0].name(), mesocycle_1.name());
        assert_eq!(mesocycles[1].id(), mesocycle_2.id());
        assert_eq!(mesocycles[1].name(), mesocycle_2.name());
    }
}
