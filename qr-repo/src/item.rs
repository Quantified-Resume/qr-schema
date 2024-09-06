use crate::{
    core::{check_table_exist, Column, ColumnType, Table},
    util::select_one_row,
};
use qr_model::Item;
use rusqlite::{params, Connection};
use serde_json;

const TABLE_NAME: &str = "bucket";

fn table_def() -> Table {
    Table {
        name: TABLE_NAME.to_string(),
        columns: vec![
            Column::required("bucket_id", ColumnType::Integer),
            Column::required("ref_id", ColumnType::Text),
            Column::required("type", ColumnType::Text),
            Column::required("timestamp", ColumnType::TimeStamp),
            Column::nullable("duration", ColumnType::Integer),
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

pub fn exist_item_by_bucket_id(conn: &Connection, bucket_id: i64) -> bool {
    let sql = format!("SELECT 1 from {} WHERE bucket_id = ? limit 1", TABLE_NAME);
    match select_one_row(conn, &sql, [bucket_id], |_| Ok(())) {
        Ok(r) => r.is_some(),
        Err(e) => {
            println!("Failed to find bucket by id: {}", e);
            false
        }
    }
}

pub fn insert_item(conn: &Connection, bucket_id: i64, item: &Item) -> rusqlite::Result<i64> {
    let sql = format!(
        "INSERT INTO {} (bucket_id, ref_id, type, timestamp, duration, name, action, metrics, payload) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        TABLE_NAME
    );
    let mut stmt = conn.prepare(&sql)?;
    stmt.execute(params![
        bucket_id,
        item.ref_id,
        item.type_.as_ref(),
        item.timestamp,
        item.duration,
        item.name,
        item.action,
        match &item.metrics {
            Some(metrics) => serde_json::to_string(metrics).unwrap(),
            None => "{}".to_string(),
        },
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
