use rusqlite::{Connection, params, params_from_iter};

/// Renumbers the `position` column of `table` so the rows in `ordered_ids` take
/// positions 0, 1, 2, … in that order, within the parent scope identified by
/// every `(column, id)` pair in `scope` (all matched with `AND`).
///
/// Returns `Ok(false)` without changing anything when `ordered_ids` is not
/// exactly the current set of children in that scope, leaving callers to
/// surface their own mismatch error.
///
/// `table` and the scope column names are always crate-internal string
/// literals, never caller input, so interpolating them into the SQL is not an
/// injection vector.
pub(super) fn reorder(
    conn: &mut Connection,
    table: &str,
    scope: &[(&str, i64)],
    ordered_ids: &[i64],
) -> rusqlite::Result<bool> {
    let where_clause = scope
        .iter()
        .map(|(column, _)| format!("{column} = ?"))
        .collect::<Vec<_>>()
        .join(" AND ");
    let scope_ids = || params_from_iter(scope.iter().map(|(_, id)| *id));

    let tx = conn.transaction()?;

    let mut current_ids = {
        let mut stmt = tx.prepare(&format!("SELECT id FROM {table} WHERE {where_clause}"))?;
        stmt.query_map(scope_ids(), |row| row.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<i64>>>()?
    };

    let mut wanted = ordered_ids.to_vec();
    current_ids.sort_unstable();
    wanted.sort_unstable();
    if current_ids != wanted {
        return Ok(false);
    }

    // Move every row out of the [0, n) target range into distinct negatives
    // first, so reassigning final positions can never transiently collide with
    // the UNIQUE(scope, position) constraint mid-statement.
    tx.execute(
        &format!("UPDATE {table} SET position = -1 - position WHERE {where_clause}"),
        scope_ids(),
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
