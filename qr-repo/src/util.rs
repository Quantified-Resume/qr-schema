use rusqlite::{params_from_iter, types::Value, Connection, Params, Row};

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
    let mut stmt = conn.prepare(sql)?;
    let opt = match stmt.query_row(params, f) {
        Ok(val) => Some(val),
        // Ignored no row error, return None
        Err(e) => match "Query returned no rows" == e.to_string().to_string() {
            true => None,
            false => return Err(e),
        },
    };
    Ok(opt)
}

pub fn select_all_rows<T, F>(
    conn: &Connection,
    table_name: &str,
    clauses: Vec<String>,
    params: Vec<Value>,
    f: F,
) -> rusqlite::Result<Vec<T>>
where
    F: Fn(&Row<'_>) -> rusqlite::Result<T>,
{
    let mut real_clauses = clauses.clone();
    real_clauses.insert(0, String::from("true"));
    let sql = format!(
        "SELECT * FROM {} WHERE {}",
        table_name,
        real_clauses.join(" AND ")
    );

    let mut stmt = conn.prepare(&sql)?;

    let sql_params = params_from_iter(params);
    let mut rows = stmt.query(sql_params)?;
    let mut res: Vec<T> = Vec::new();
    while let Some(row) = rows.next()? {
        res.push(f(row)?);
    }
    Ok(res)
}
