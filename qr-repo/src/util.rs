use rusqlite::{Connection, Params, Row};

/**
 * Select one row as sql
 *
 * Returns none if no row selected
 */
pub fn select_one_row<T, P, F>(conn: &Connection, sql: &str, params: P, f: F) -> Option<T>
where
    P: Params,
    F: FnOnce(&Row<'_>) -> rusqlite::Result<T>,
{
    let mut statement = conn.prepare(sql).expect("Failed to open statement");
    let row_res = statement.query_row(params, f);
    match row_res {
        Ok(val) => Some(val),
        // Ignored no row error, return None
        Err(e) => match "Query returned no rows" == e.to_string().to_string() {
            true => None,
            false => panic!("{}", e),
        },
    }
}
