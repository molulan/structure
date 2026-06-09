use rusqlite::{Connection, params};

/// Renumbers the `position` column of `table` so the rows in `ordered_ids` take
/// positions 0, 1, 2, … in that order, within the parent identified by
/// `parent_column = parent_id`.
///
/// Returns `Ok(false)` without changing anything when `ordered_ids` is not
/// exactly the current set of children, leaving callers to surface their own
/// mismatch error.
///
/// `table` and `parent_column` are always crate-internal string literals, never
/// caller input, so interpolating them into the SQL is not an injection vector.
pub(super) fn reorder(
    conn: &mut Connection,
    table: &str,
    parent_column: &str,
    parent_id: i64,
    ordered_ids: &[i64],
) -> rusqlite::Result<bool> {
    let tx = conn.transaction()?;

    let mut current = {
        let mut stmt = tx.prepare(&format!(
            "SELECT id FROM {table} WHERE {parent_column} = ?1"
        ))?;
        stmt.query_map(params![parent_id], |row| row.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<i64>>>()?
    };

    let mut wanted = ordered_ids.to_vec();
    current.sort_unstable();
    wanted.sort_unstable();
    if current != wanted {
        return Ok(false);
    }

    // Move every row out of the [0, n) target range into distinct negatives
    // first, so reassigning final positions can never transiently collide with
    // the UNIQUE(parent, position) constraint mid-statement.
    tx.execute(
        &format!("UPDATE {table} SET position = -1 - position WHERE {parent_column} = ?1"),
        params![parent_id],
    )?;

    for (index, id) in ordered_ids.iter().enumerate() {
        tx.execute(
            &format!("UPDATE {table} SET position = ?1 WHERE id = ?2"),
            params![index as i64, id],
        )?;
    }

    tx.commit()?;
    Ok(true)
}
