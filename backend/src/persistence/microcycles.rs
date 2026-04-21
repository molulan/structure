use crate::domain::planning::Microcycle;
use rusqlite::{Connection, Result, params};

pub fn create_microcycles_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS microcycles (
            id INTEGER PRIMARY KEY,
            mesocycle_id INTEGER NOT NULL REFERENCES mesocycles(id),
            position INTEGER NOT NULL
        )",
        (),
    )?;
    Ok(())
}

pub fn create_microcycle(conn: &Connection, mesocycle_id: i64) -> Result<Microcycle> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM microcycles WHERE mesocycle_id = ?1",
        [mesocycle_id],
        |row| row.get(0),
    )?;

    let position = count as u32;

    conn.execute(
        "INSERT INTO microcycles (mesocycle_id, position) VALUES (?1, ?2)",
        params![mesocycle_id, position],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Microcycle::new(id, position))
}

pub fn list_microcycles(conn: &Connection, mesocycle_id: i64) -> Result<Vec<Microcycle>> {
    let mut stmt = conn.prepare(
        "SELECT id, position FROM microcycles WHERE mesocycle_id = ?1 ORDER BY position ASC",
    )?;

    stmt.query_map([mesocycle_id], |row| {
        let id = row.get(0)?;
        let position: i64 = row.get(1)?;
        let position = position as u32;
        Ok(Microcycle::new(id, position))
    })?
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{mesocycles::create_mesocycle, sqlite};

    fn setup_test_db() -> Connection {
        sqlite::init_db(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn create_microcycle_generates_microcycle_with_position_0_in_empty_mesocycle() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "hypertrophy").unwrap();

        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        assert_eq!(microcycle.position(), 0);
    }

    #[test]
    fn multiple_microcycles_in_same_mesocycle_gets_increasing_position_numbers() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "hypertrophy").unwrap();

        let microcycle_1 = create_microcycle(&conn, mesocycle.id()).unwrap();
        let microcycle_2 = create_microcycle(&conn, mesocycle.id()).unwrap();
        let microcycle_3 = create_microcycle(&conn, mesocycle.id()).unwrap();

        assert_eq!(microcycle_1.position(), 0);
        assert_eq!(microcycle_2.position(), 1);
        assert_eq!(microcycle_3.position(), 2);
    }

    #[test]
    fn multiple_microcycles_in_same_mesocycle_gets_unique_ids() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "hypertrophy").unwrap();

        let microcycle_1 = create_microcycle(&conn, mesocycle.id()).unwrap();
        let microcycle_2 = create_microcycle(&conn, mesocycle.id()).unwrap();

        assert_ne!(microcycle_1.id(), microcycle_2.id());
    }

    #[test]
    fn created_microcycle_appear_in_list_with_correct_id_and_position() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "hypertrophy").unwrap();

        let microcycle = create_microcycle(&conn, mesocycle.id()).unwrap();

        let result = list_microcycles(&conn, mesocycle.id()).unwrap();

        assert_eq!(microcycle.id(), result[0].id());
        assert_eq!(microcycle.position(), result[0].position());
    }

    #[test]
    fn multiple_microcycles_appear_in_list_with_correct_id_and_position() {
        let conn = setup_test_db();

        let mesocycle = create_mesocycle(&conn, "hypertrophy").unwrap();

        let microcycle_1 = create_microcycle(&conn, mesocycle.id()).unwrap();
        let microcycle_2 = create_microcycle(&conn, mesocycle.id()).unwrap();

        let result = list_microcycles(&conn, mesocycle.id()).unwrap();

        assert_eq!(microcycle_1.id(), result[0].id());
        assert_eq!(microcycle_1.position(), result[0].position());
        assert_eq!(microcycle_2.id(), result[1].id());
        assert_eq!(microcycle_2.position(), result[1].position());
    }

    #[test]
    fn microcycles_are_scoped_to_their_parent_mesocycle() {
        let conn = setup_test_db();

        let mesocycle_1 = create_mesocycle(&conn, "hypertrophy").unwrap();
        let microcycle_1 = create_microcycle(&conn, mesocycle_1.id()).unwrap();

        let mesocycle_2 = create_mesocycle(&conn, "strength").unwrap();
        let microcycle_2 = create_microcycle(&conn, mesocycle_2.id()).unwrap();

        let result_1 = list_microcycles(&conn, mesocycle_1.id()).unwrap();
        assert_eq!(result_1.len(), 1);
        assert_eq!(result_1[0].id(), microcycle_1.id());

        let result_2 = list_microcycles(&conn, mesocycle_2.id()).unwrap();
        assert_eq!(result_2.len(), 1);
        assert_eq!(result_2[0].id(), microcycle_2.id());
    }
}
