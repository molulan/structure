use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::domain::planning::{Mesocycle, MesocycleMode};

#[derive(Debug, thiserror::Error)]
pub enum MesocycleError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("mesocycle {id} not found")]
    NotFound { id: i64 },
}

pub struct MesocycleRow {
    pub id: i64,
    pub name: String,
    pub mode: MesocycleMode,
    pub microcycle_count: u32,
}

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
        params![name, mode.to_string()],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Mesocycle::new(id, name, mode))
}

pub fn get_mesocycle(conn: &Connection, id: i64) -> Result<Option<MesocycleRow>> {
    conn.query_row(
        "SELECT meso.id, meso.name, meso.mode, COUNT(micro.id) as microcycle_count
        FROM mesocycles meso
        LEFT JOIN microcycles micro ON micro.mesocycle_id = meso.id
        WHERE meso.id = ?1
        GROUP BY meso.id",
        [id],
        |row| {
            let id = row.get(0)?;
            let name: String = row.get(1)?;
            let mode: String = row.get(2)?;
            let mode = mesocycle_mode_from_str(&mode);
            let count: i64 = row.get(3)?;
            let microcycle_count = u32::try_from(count).expect(
                "COUNT(*) is always non-negative and no program will have 4 billion microcycles",
            );
            Ok(MesocycleRow {
                id,
                name,
                mode,
                microcycle_count,
            })
        },
    )
    .optional()
}

pub fn list_mesocycles(conn: &Connection) -> Result<Vec<MesocycleRow>> {
    let mut stmt = conn.prepare(
        "SELECT meso.id, meso.name, meso.mode, COUNT(micro.id) as microcycle_count
         FROM mesocycles meso
         LEFT JOIN microcycles micro ON micro.mesocycle_id = meso.id
         GROUP BY meso.id
         ORDER BY meso.id ASC",
    )?;

    stmt.query_map([], |row| {
        let id = row.get(0)?;
        let name: String = row.get(1)?;
        let mode: String = row.get(2)?;
        let mode = mesocycle_mode_from_str(&mode);
        let count: i64 = row.get(3)?;
        let microcycle_count = u32::try_from(count).expect(
            "COUNT(*) is always non-negative and no program will have 4 billion microcycles",
        );
        Ok(MesocycleRow {
            id,
            name,
            mode,
            microcycle_count,
        })
    })?
    .collect()
}

pub fn update_mesocycle(
    conn: &Connection,
    id: i64,
    name: &str,
    mode: MesocycleMode,
) -> Result<Mesocycle, MesocycleError> {
    let updated = conn.execute(
        "UPDATE mesocycles SET name = ?1, mode = ?2 WHERE id = ?3",
        params![name, mode.to_string(), id],
    )?;

    if updated == 0 {
        return Err(MesocycleError::NotFound { id });
    }

    Ok(Mesocycle::new(id, name, mode))
}

pub fn delete_mesocycle(conn: &Connection, id: i64) -> Result<(), MesocycleError> {
    let deleted = conn.execute("DELETE FROM mesocycles WHERE id = ?1", [id])?;

    if deleted == 0 {
        return Err(MesocycleError::NotFound { id });
    }

    Ok(())
}

fn mesocycle_mode_from_str(s: &str) -> MesocycleMode {
    match s {
        "Algorithmic" => MesocycleMode::Algorithmic,
        "Manual" => MesocycleMode::Manual,
        other => panic!("Unexpected mesocyle mode: {}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::connection;

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("Failed to create test database")
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
        let target = create_mesocycle(&conn, "BIG ARMS", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        let result = get_mesocycle(&conn, target.id())
            .expect("DB query should not fail")
            .expect("mesocycle should exist");

        assert_eq!(target.id(), result.id);
        assert_eq!(target.name(), result.name);
        assert_eq!(target.mode(), result.mode);
        assert_eq!(result.microcycle_count, 0);
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

        let mesocycle = create_mesocycle(&conn, "hypertrophy 1", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        let result =
            list_mesocycles(&conn).expect("listing mesocycles for a valid id should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, mesocycle.id());
        assert_eq!(result[0].name, mesocycle.name());
        assert_eq!(result[0].mode, mesocycle.mode());
        assert_eq!(result[0].microcycle_count, 0);
    }

    #[test]
    fn multiple_mesocycles_get_unique_ids_and_appear_in_list() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle_1 =
            create_mesocycle(&conn, "Big Arms", mode).expect("mesocycle creation should succeed");
        let mesocycle_2 = create_mesocycle(&conn, "Bigger Arms!", mode)
            .expect("mesocycle creation should succeed");
        assert_ne!(mesocycle_1.id(), mesocycle_2.id());

        let mesocycles =
            list_mesocycles(&conn).expect("listing mesocycles for a valid id should succeed");
        assert_eq!(mesocycles.len(), 2);
        assert_eq!(mesocycles[0].id, mesocycle_1.id());
        assert_eq!(mesocycles[0].name, mesocycle_1.name());
        assert_eq!(mesocycles[1].id, mesocycle_2.id());
        assert_eq!(mesocycles[1].name, mesocycle_2.name());
    }

    #[test]
    fn list_mesocycles_includes_correct_microcycle_count() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        crate::persistence::microcycles::create_microcycle(&conn, mesocycle.id())
            .expect("microcycle creation should succeed");
        crate::persistence::microcycles::create_microcycle(&conn, mesocycle.id())
            .expect("microcycle creation should succeed");

        let result = list_mesocycles(&conn).expect("listing should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].microcycle_count, 2);
    }

    #[test]
    fn get_mesocycle_includes_correct_microcycle_count() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        crate::persistence::microcycles::create_microcycle(&conn, mesocycle.id())
            .expect("microcycle creation should succeed");

        let result = get_mesocycle(&conn, mesocycle.id())
            .expect("query should succeed")
            .expect("mesocycle should exist");

        assert_eq!(result.microcycle_count, 1);
    }

    #[test]
    fn update_mesocycle_changes_name_and_mode() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        let updated = update_mesocycle(
            &conn,
            mesocycle.id(),
            "strength",
            MesocycleMode::Algorithmic,
        )
        .expect("update should succeed");

        assert_eq!(updated.name(), "strength");
        assert_eq!(updated.mode(), MesocycleMode::Algorithmic);

        let persisted = get_mesocycle(&conn, mesocycle.id())
            .expect("query should succeed")
            .expect("mesocycle should exist");
        assert_eq!(persisted.name, "strength");
        assert_eq!(persisted.mode, MesocycleMode::Algorithmic);
    }

    #[test]
    fn update_mesocycle_returns_not_found_when_mesocycle_does_not_exist() {
        let conn = setup_test_db();

        let result = update_mesocycle(&conn, 1234, "strength", MesocycleMode::Manual);

        assert!(matches!(result, Err(MesocycleError::NotFound { id: 1234 })));
    }

    #[test]
    fn delete_mesocycle_removes_it() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        delete_mesocycle(&conn, mesocycle.id()).expect("delete should succeed");

        let result = get_mesocycle(&conn, mesocycle.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn delete_mesocycle_returns_not_found_when_mesocycle_does_not_exist() {
        let conn = setup_test_db();

        let result = delete_mesocycle(&conn, 1234);

        assert!(matches!(result, Err(MesocycleError::NotFound { id: 1234 })));
    }

    #[test]
    fn delete_mesocycle_cascades_to_its_microcycles() {
        let conn = setup_test_db();
        let mesocycle = create_mesocycle(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let microcycle = crate::persistence::microcycles::create_microcycle(&conn, mesocycle.id())
            .expect("microcycle creation should succeed");

        delete_mesocycle(&conn, mesocycle.id()).expect("delete should succeed");

        let orphan = crate::persistence::microcycles::get_microcycle(&conn, microcycle.id())
            .expect("query should succeed");
        assert!(orphan.is_none());
    }
}
