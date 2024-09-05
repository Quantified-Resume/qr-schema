use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use qr_model::{Bucket, BucketStatus, Builtin};
use rusqlite::{params, Connection, Row};
use serde_json;

use crate::{
    convert::{date_from_sql, json_map_from_sql, str_from_sql},
    core::{check_table_exist, Column, ColumnType, Table},
    util::{self, select_one_row},
};

const TABLE_NAME: &str = "bucket";

fn table_def() -> Table {
    Table {
        name: TABLE_NAME.to_string(),
        columns: vec![
            Column::required("no", ColumnType::Integer),
            Column::required("name", ColumnType::Text),
            Column::nullable("builtin", ColumnType::Text),
            Column::nullable("builtin_ref_id", ColumnType::Text), // Ref ID of builtin channel
            Column::nullable("status", ColumnType::Text),
            Column::nullable("desc", ColumnType::Text),
            Column::nullable("url", ColumnType::Text), // Reserved field
            Column::nullable("payload", ColumnType::Json),
            Column::with_default("built_in_flag", ColumnType::Bool, "false"), // Reserved field
        ],
    }
}

pub fn init_table(conn: &Connection) {
    check_table_exist(conn, TABLE_NAME, table_def);
}

pub fn insert_bucket(conn: &Connection, bucket: &Bucket) -> rusqlite::Result<i64> {
    let sql = format!(
        "INSERT INTO {} (no, name, status, desc, url, payload, builtin, builtin_ref_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        TABLE_NAME
    );
    let mut statement = conn.prepare(&sql)?;
    statement
        .execute(params![
            bucket.no,
            bucket.name,
            bucket.status.as_ref(),
            bucket.desc,
            bucket.url,
            match &bucket.payload {
                Some(val) => serde_json::to_string(val).unwrap(),
                None => "{}".to_string(),
            },
            bucket.builtin,
            bucket.builtin_ref_id,
        ])
        .map(|_| conn.last_insert_rowid())
}

pub fn select_bucket(conn: &Connection, id: i64) -> Option<Bucket> {
    let sql = format!("SELECT * FROM {} WHERE id = :id limit 1", TABLE_NAME);
    util::select_one_row(conn, &sql, &[(":id", &id.to_string())], map_row)
}

fn map_row(row: &Row<'_>) -> rusqlite::Result<Bucket> {
    let builtin = str_from_sql(row.get("builtin")).and_then(|val| Builtin::from_str(&val).ok());
    let status = str_from_sql(row.get("status"))
        .and_then(|val: String| BucketStatus::from_str(&val).ok())
        .unwrap_or_default();
    Ok(Bucket {
        id: row.get_unwrap("id"),
        no: row.get_unwrap("no"),
        name: row.get_unwrap("name"),
        builtin,
        builtin_ref_id: row.get("builtin_ref_id").ok(),
        status,
        desc: row.get_unwrap("desc"),
        url: row.get_unwrap("url"),
        tag: None,
        created: Some(date_from_sql(row.get("create_time")).expect("Failed to query create_time")),
        last_modified: Some(
            date_from_sql(row.get("modify_time")).expect("Failed to query modify_time"),
        ),
        payload: json_map_from_sql(row.get("payload")),
    })
}

pub fn select_bucket_by_builtin(
    conn: &Connection,
    builtin: &Builtin,
    builtin_ref_id: Option<&str>,
) -> Option<Bucket> {
    let b_str = builtin.as_ref();
    match builtin_ref_id {
        Some(ref_id) => {
            let sql = format!(
                "SELECT * FROM {} WHERE builtin = :b and builtin_ref_id = :rid LIMIT 1",
                TABLE_NAME
            );
            select_one_row(conn, &sql, &[(":b", b_str), (":rid", ref_id)], map_row)
        }
        None => {
            let sql = format!(
                "SELECT * FROM {} WHERE builtin = :b and builtin_ref_id IS NULL LIMIT 1",
                TABLE_NAME
            );
            select_one_row(conn, &sql, &[(":b", b_str)], map_row)
        }
    }
}

pub fn select_all_buckets(conn: &Connection) -> Vec<Bucket> {
    let sql: String = format!("SELECT * FROM {}", TABLE_NAME);
    let mut stmt = conn.prepare(&sql).unwrap();
    let mut rows = stmt.query([]).unwrap();
    let mut res: Vec<Bucket> = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let b = map_row(row).unwrap();
        res.push(b);
    }

    res
}

pub fn update_bucket(conn: &Connection, bucket: &Bucket) -> rusqlite::Result<()> {
    let sql = format!(
        "UPDATE {} SET modify_time = ?, name = ?, desc = ? WHERE id = ?",
        TABLE_NAME
    );
    let mut stmt = conn.prepare(&sql)?;
    stmt.execute(params![
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string(),
        bucket.name,
        bucket.desc,
        bucket.id
    ])
    .map(|_| ())
}

pub fn delete_bucket(conn: &Connection, bucket_id: i64) -> rusqlite::Result<()> {
    let sql = format!("DELETE FROM {} where id = ?1", TABLE_NAME);
    conn.execute(&sql, [bucket_id]).map(|_| ())
}

#[test]
fn test() {
    use chrono::DateTime;
    use serde_json::{to_string, Map, Number, Value};

    let conn = Connection::open_in_memory().unwrap();
    init_table(&conn);

    let mut payload = Map::new();
    payload.insert("test".to_string(), Value::String("fff".to_string()));
    payload.insert("a1".to_string(), Value::Number(Number::from(123)));

    println!("{}", to_string(&payload).unwrap());

    let bucket = Bucket {
        id: None,
        no: Some(234),
        name: "foobar".to_string(),
        builtin: Some(Builtin::BrowserTime),
        builtin_ref_id: None,
        status: BucketStatus::Enabled,
        desc: Some("Mock description".to_string()),
        url: Some("https://qr.com".to_string()),
        tag: None,
        created: Some(DateTime::default()),
        last_modified: Some(DateTime::default()),
        payload: Some(payload),
    };
    let id = insert_bucket(&conn, &bucket).unwrap();
    assert!(id > 0);
    println!("Bucket inserted: id={}", id);

    let mut bucket_res = select_bucket(&conn, id);
    let b = bucket_res.unwrap();
    assert_eq!(b.id.clone().unwrap(), 1);
    assert_eq!(b.no.clone().unwrap(), 234);
    assert_eq!(b.name, "foobar".to_string());
    assert_eq!(b.desc.clone().unwrap(), "Mock description".to_string());
    assert_eq!(b.url.clone().unwrap(), "https://qr.com".to_string());
    println!("{}", b);

    bucket_res = select_bucket(&conn, 2);
    assert!(bucket_res.is_none());
}
