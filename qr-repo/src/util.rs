use rusqlite::{Connection, Params, Row};

/**
 * Select one row as sql
 *
 * Returns none if no row selected
 */
pub fn select_one_row<T, P, F>(
    conn: &Connection,
    sql: &str,
    params: P,
    f: F,
) -> rusqlite::Result<Option<T>>
where
    P: Params,
    F: FnOnce(&Row<'_>) -> rusqlite::Result<T>,
{
    let mut statement = match conn.prepare(sql) {
        Ok(v) => v,
        Err(e) => return rusqlite::Result::Err(e),
    };
    match statement.query_row(params, f) {
        Ok(val) => Ok(Some(val)),
        // Ignored no row error, return None
        Err(e) => match "Query returned no rows" == e.to_string().to_string() {
            true => Ok(None),
            false => Err(e),
        },
    }
}
