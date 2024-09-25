use crate::{
    convert::json_map_from_sql,
    core::{check_table_exist, Column, ColumnType, Table},
    util::{select_all_rows, select_one_row},
};
use qr_model::Item;
use rusqlite::{params, types::Value, Connection, Row};
use serde_json::{self};

const TABLE_NAME: &str = "item";

fn table_def() -> Table {
    Table {
        name: TABLE_NAME.to_string(),
        columns: vec![
            Column::required("bucket_id", ColumnType::Integer),
            Column::required("ref_id", ColumnType::Text),
            Column::required("timestamp", ColumnType::TimeStamp),
            Column::nullable("name", ColumnType::Text),
            Column::required("action", ColumnType::Text),
            Column::nullable("metrics", ColumnType::Json),
            Column::nullable("payload", ColumnType::Json),
        ],
    }
}

pub fn init_table(conn: &Connection) -> rusqlite::Result<bool> {
    check_table_exist(conn, TABLE_NAME, table_def)
}

pub fn exist_item_by_bucket_id(conn: &Connection, bucket_id: i64) -> rusqlite::Result<bool> {
    let sql = format!("SELECT 1 from {} WHERE bucket_id = ? limit 1", TABLE_NAME);
    let exist = select_one_row(conn, &sql, [bucket_id], |_| Ok(()))?;
    Ok(exist.is_some())
}

pub fn insert_item(conn: &Connection, bucket_id: i64, item: &Item) -> rusqlite::Result<i64> {
    log::debug!("Insert item: {:?}", item);
    let sql = format!(
        "INSERT INTO {} (bucket_id, ref_id, timestamp, name, action, metrics, payload) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        TABLE_NAME
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    stmt.execute(params![
        bucket_id,
        item.ref_id,
        item.timestamp,
        item.name,
        item.action,
        serde_json::to_string(&item.metrics).unwrap(),
        match &item.payload {
            Some(payload) => serde_json::to_string(payload).unwrap(),
            None => "{}".to_string(),
        },
    ])
    .map(|_| conn.last_insert_rowid())
}

pub fn delete_item_by_bucket_id(conn: &Connection, bucket_id: i64) -> rusqlite::Result<i32> {
    let sql = format!("DELETE FROM {} WHERE bucket_id = ?", TABLE_NAME);
    conn.execute(&sql, [bucket_id]).map(|res| res as i32)
}

pub fn select_item_by_bid_and_rid(
    conn: &Connection,
    bucket_id: i64,
    ref_id: &str,
) -> rusqlite::Result<Option<Item>> {
    let sql = format!(
        "SELECT * FROM {} WHERE bucket_id = ?1 and ref_id = ?2 LIMIT 1",
        TABLE_NAME
    );

    select_one_row(conn, &sql, params![bucket_id, ref_id], map_row)
}

fn map_row(row: &Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        id: row.get_unwrap("id"),
        ref_id: row.get_unwrap("ref_id"),
        timestamp: row.get_unwrap("timestamp"),
        name: row.get_unwrap("name"),
        action: row.get_unwrap("action"),
        metrics: json_map_from_sql(row.get("metrics")).unwrap(),
        payload: json_map_from_sql(row.get("payload")),
    })
}

pub fn select_all_items(
    conn: &Connection,
    clauses: Vec<String>,
    params: Vec<Value>,
) -> rusqlite::Result<Vec<Item>> {
    log::info!("QUERY ITEMS: clauses={:?}, params={:?}", clauses, params);
    select_all_rows(conn, TABLE_NAME, clauses, params, map_row)
}

pub fn select_item_by_ref_id(
    conn: &Connection,
    bucket_id: i64,
    ref_id: &str,
) -> rusqlite::Result<Option<Item>> {
    let sql = format!(
        "SELECT * FROM {} WHERE bucket_id = ?1 AND ref_id = ?2",
        TABLE_NAME
    );
    select_one_row(conn, &sql, params![bucket_id, ref_id], map_row)
}
