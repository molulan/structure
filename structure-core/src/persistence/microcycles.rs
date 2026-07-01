use crate::domain::planning::{Microcycle, Phase};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, thiserror::Error)]
pub enum MicrocycleError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("associated mesocycle {id} not found")]
    AssociatedMesocycleNotFound { id: i64 },
    #[error("microcycle {id} not found")]
    NotFound { id: i64 },
    #[error("reorder list does not match the microcycles of mesocycle {mesocycle_id}")]
    ReorderMismatch { mesocycle_id: i64 },
}

pub(super) fn create_microcycles_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS microcycles (
            id INTEGER PRIMARY KEY,
            mesocycle_id INTEGER NOT NULL REFERENCES mesocycles(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            phase TEXT CHECK(
                phase IN ('Accumulation', 'Intensification', 'Deload')
            ),
            UNIQUE(mesocycle_id, position)
        )",
        (),
    )?;
    Ok(())
}

fn mesocycle_exists(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM mesocycles WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn create(conn: &Connection, mesocycle_id: i64) -> Result<Microcycle, MicrocycleError> {
    if !mesocycle_exists(conn, mesocycle_id)? {
        return Err(MicrocycleError::AssociatedMesocycleNotFound { id: mesocycle_id });
    }

    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM microcycles WHERE mesocycle_id = ?1",
        [mesocycle_id],
        |row| row.get(0),
    )?;

    let position = u32::try_from(next_position).expect(
        "positions are non-negative and no training program will have 4 billion microcycles",
    );

    conn.execute(
        "INSERT INTO microcycles (mesocycle_id, position) VALUES (?1, ?2)",
        params![mesocycle_id, position],
    )?;

    let id = conn.last_insert_rowid();

    Ok(Microcycle::new(id, position, None))
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Microcycle>> {
    conn.query_row(
        "SELECT id, position, phase FROM microcycles WHERE id = ?1",
        [id],
        |row| {
            let id = row.get(0)?;
            let position: i64 = row.get(1)?;
            let position =
                u32::try_from(position).expect("position stored in DB was originally a u32");
            let phase: Option<String> = row.get(2)?;
            let phase = phase.as_deref().map(phase_from_str);
            Ok(Microcycle::new(id, position, phase))
        },
    )
    .optional()
}

pub fn update_phase(
    conn: &Connection,
    id: i64,
    phase: Option<Phase>,
) -> Result<(), MicrocycleError> {
    let updated = conn.execute(
        "UPDATE microcycles SET phase = ?2 WHERE id = ?1",
        params![id, phase.map(|p| p.as_str())],
    )?;

    if updated == 0 {
        return Err(MicrocycleError::NotFound { id });
    }

    Ok(())
}

pub fn list(conn: &Connection, mesocycle_id: i64) -> Result<Vec<Microcycle>, MicrocycleError> {
    if !mesocycle_exists(conn, mesocycle_id)? {
        return Err(MicrocycleError::AssociatedMesocycleNotFound { id: mesocycle_id });
    }

    let mut stmt = conn.prepare(
        "SELECT id, position, phase FROM microcycles WHERE mesocycle_id = ?1 ORDER BY position ASC",
    )?;

    stmt.query_map([mesocycle_id], |row| {
        let id = row.get(0)?;
        let position: i64 = row.get(1)?;
        let position = u32::try_from(position).expect("position stored in DB was originally a u32");
        let phase: Option<String> = row.get(2)?;
        let phase = phase.as_deref().map(phase_from_str);
        Ok(Microcycle::new(id, position, phase))
    })?
    .map(|result| result.map_err(MicrocycleError::from))
    .collect()
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), MicrocycleError> {
    let deleted = conn.execute("DELETE FROM microcycles WHERE id = ?1", [id])?;

    if deleted == 0 {
        return Err(MicrocycleError::NotFound { id });
    }

    Ok(())
}

pub fn reorder(
    conn: &mut Connection,
    mesocycle_id: i64,
    ordered_ids: &[i64],
) -> Result<(), MicrocycleError> {
    let matched = super::positions::reorder(
        conn,
        "microcycles",
        "mesocycle_id",
        mesocycle_id,
        ordered_ids,
    )?;

    if matched {
        Ok(())
    } else {
        Err(MicrocycleError::ReorderMismatch { mesocycle_id })
    }
}

fn phase_from_str(s: &str) -> Phase {
    match s {
        "Accumulation" => Phase::Accumulation,
        "Intensification" => Phase::Intensification,
        "Deload" => Phase::Deload,
        other => panic!("Unexpected microcycle phase: {}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::planning::MesocycleMode,
        persistence::{connection, mesocycles},
    };

    fn setup_test_db() -> Connection {
        connection::init_db(":memory:").expect("Failed to create test database")
    }

    #[test]
    fn get_microcycle_returns_none_on_invalid_id() {
        let conn = setup_test_db();

        let result = get(&conn, 1234).expect("Should return None");

        assert!(result.is_none());
    }

    #[test]
    fn get_microcycle_returns_correct_microcycle() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle_1 = mesocycles::create(&conn, "small arms", mode)
            .expect("Should be able to create mesocycle");

        let mesocycle_2 = mesocycles::create(&conn, "BIG ARMS", mode)
            .expect("Should be able to create mesocycle");

        let _ = create(&conn, mesocycle_1.id()).expect("Should be able to create microcycle");

        let target = create(&conn, mesocycle_2.id()).expect("Should be able to create microcycle");

        let result = get(&conn, target.id())
            .expect("DB query should not fail")
            .expect("microcycle should exist");

        assert_eq!(target, result);
    }

    #[test]
    fn list_microcycles_returns_error_when_called_with_invalid_mesocycle_id() {
        let conn = setup_test_db();

        let result = list(&conn, 1234);

        assert!(result.is_err());
    }

    #[test]
    fn list_microcycles_returns_empty_list_for_mesocycle_with_no_microcycles() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle = mesocycles::create(&conn, "welcome to the gunshow", mode)
            .expect("mesocycle creation should succeed");

        let result =
            list(&conn, mesocycle.id()).expect("listing microcycles for a valid id should succeed");

        assert!(result.is_empty());
    }

    #[test]
    fn microcycle_is_created_without_a_phase() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        let microcycle = create(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        assert_eq!(microcycle.phase(), None);
    }

    #[test]
    fn update_phase_assigns_a_phase_that_is_read_back() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let microcycle = create(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        update_phase(&conn, microcycle.id(), Some(Phase::Intensification))
            .expect("update_phase should succeed");

        let result = get(&conn, microcycle.id())
            .expect("query should succeed")
            .expect("microcycle should exist");
        assert_eq!(result.phase(), Some(Phase::Intensification));
    }

    #[test]
    fn update_phase_to_none_clears_an_existing_phase() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let microcycle = create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        update_phase(&conn, microcycle.id(), Some(Phase::Deload))
            .expect("update_phase should succeed");

        update_phase(&conn, microcycle.id(), None).expect("clearing the phase should succeed");

        let result = get(&conn, microcycle.id())
            .expect("query should succeed")
            .expect("microcycle should exist");
        assert_eq!(result.phase(), None);
    }

    #[test]
    fn update_phase_returns_not_found_when_microcycle_does_not_exist() {
        let conn = setup_test_db();

        let result = update_phase(&conn, 1234, Some(Phase::Accumulation));

        assert!(matches!(
            result,
            Err(MicrocycleError::NotFound { id: 1234 })
        ));
    }

    #[test]
    fn create_microcycle_generates_microcycle_with_position_0_in_empty_mesocycle() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle = mesocycles::create(&conn, "hypertrophy", mode)
            .expect("mesocycle creation should succeed");

        let microcycle = create(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        assert_eq!(microcycle.position(), 0);
    }

    #[test]
    fn multiple_microcycles_in_same_mesocycle_gets_increasing_position_numbers() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "hypertrophy", mode)
            .expect("mesocycle creation should succeed");

        let microcycle_1 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let microcycle_2 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let microcycle_3 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        assert_eq!(microcycle_1.position(), 0);
        assert_eq!(microcycle_2.position(), 1);
        assert_eq!(microcycle_3.position(), 2);
    }

    #[test]
    fn multiple_microcycles_in_same_mesocycle_gets_unique_ids() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle = mesocycles::create(&conn, "hypertrophy", mode)
            .expect("mesocycle creation should succeed");

        let microcycle_1 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let microcycle_2 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        assert_ne!(microcycle_1.id(), microcycle_2.id());
    }

    #[test]
    fn created_microcycle_appear_in_list_with_correct_id_and_position() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Algorithmic;

        let mesocycle = mesocycles::create(&conn, "hypertrophy", mode)
            .expect("mesocycle creation should succeed");

        let microcycle = create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let result =
            list(&conn, mesocycle.id()).expect("listing microcycles for a valid id should succeed");

        assert_eq!(microcycle.id(), result[0].id());
        assert_eq!(microcycle.position(), result[0].position());
    }

    #[test]
    fn multiple_microcycles_appear_in_list_with_correct_id_and_position() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle = mesocycles::create(&conn, "hypertrophy", mode)
            .expect("mesocycle creation should succeed");

        let microcycle_1 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let microcycle_2 =
            create(&conn, mesocycle.id()).expect("microcycle creation should succeed");
        let result =
            list(&conn, mesocycle.id()).expect("listing microcycles for a valid id should succeed");

        assert_eq!(microcycle_1.id(), result[0].id());
        assert_eq!(microcycle_1.position(), result[0].position());
        assert_eq!(microcycle_2.id(), result[1].id());
        assert_eq!(microcycle_2.position(), result[1].position());
    }

    #[test]
    fn microcycles_are_scoped_to_their_parent_mesocycle() {
        let conn = setup_test_db();
        let mode = MesocycleMode::Manual;

        let mesocycle_1 = mesocycles::create(&conn, "hypertrophy", mode)
            .expect("mesocycle creation should succeed");
        let microcycle_1 =
            create(&conn, mesocycle_1.id()).expect("microcycle creation should succeed");

        let mesocycle_2 =
            mesocycles::create(&conn, "strength", mode).expect("mesocycle creation should succeed");
        let microcycle_2 =
            create(&conn, mesocycle_2.id()).expect("microcycle creation should succeed");

        let result_1 = list(&conn, mesocycle_1.id())
            .expect("listing microcycles for a valid id should succeed");
        assert_eq!(result_1.len(), 1);
        assert_eq!(result_1[0].id(), microcycle_1.id());

        let result_2 = list(&conn, mesocycle_2.id())
            .expect("listing microcycles for a valid id should succeed");
        assert_eq!(result_2.len(), 1);
        assert_eq!(result_2[0].id(), microcycle_2.id());
    }

    #[test]
    fn create_microcycle_after_delete_does_not_reuse_a_position() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");

        let _first = create(&conn, mesocycle.id()).expect("creation should succeed");
        let middle = create(&conn, mesocycle.id()).expect("creation should succeed");
        let _last = create(&conn, mesocycle.id()).expect("creation should succeed");

        delete(&conn, middle.id()).expect("delete should succeed");

        // MAX(position) is still 2, so the next position is 3 — a COUNT-based
        // scheme would compute 2 and collide with the surviving last microcycle.
        let next = create(&conn, mesocycle.id()).expect("creation should succeed");
        assert_eq!(next.position(), 3);
    }

    #[test]
    fn delete_microcycle_removes_it() {
        let conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let microcycle = create(&conn, mesocycle.id()).expect("microcycle creation should succeed");

        delete(&conn, microcycle.id()).expect("delete should succeed");

        let result = get(&conn, microcycle.id()).expect("query should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn delete_microcycle_returns_not_found_when_microcycle_does_not_exist() {
        let conn = setup_test_db();

        let result = delete(&conn, 1234);

        assert!(matches!(
            result,
            Err(MicrocycleError::NotFound { id: 1234 })
        ));
    }

    #[test]
    fn reorder_microcycles_rewrites_positions_in_the_given_order() {
        let mut conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let a = create(&conn, mesocycle.id()).expect("creation should succeed");
        let b = create(&conn, mesocycle.id()).expect("creation should succeed");
        let c = create(&conn, mesocycle.id()).expect("creation should succeed");

        reorder(&mut conn, mesocycle.id(), &[c.id(), a.id(), b.id()])
            .expect("reorder should succeed");

        let ordered = list(&conn, mesocycle.id()).expect("listing should succeed");
        let ids: Vec<i64> = ordered.iter().map(|m| m.id()).collect();
        assert_eq!(ids, vec![c.id(), a.id(), b.id()]);
        assert_eq!(ordered[0].position(), 0);
        assert_eq!(ordered[1].position(), 1);
        assert_eq!(ordered[2].position(), 2);
    }

    #[test]
    fn reorder_microcycles_returns_mismatch_when_ids_do_not_match_children() {
        let mut conn = setup_test_db();
        let mesocycle = mesocycles::create(&conn, "hypertrophy", MesocycleMode::Manual)
            .expect("mesocycle creation should succeed");
        let a = create(&conn, mesocycle.id()).expect("creation should succeed");
        let _b = create(&conn, mesocycle.id()).expect("creation should succeed");

        let result = reorder(&mut conn, mesocycle.id(), &[a.id()]);

        assert!(matches!(
            result,
            Err(MicrocycleError::ReorderMismatch { .. })
        ));
    }
}
