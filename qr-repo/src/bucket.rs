use crate::{
    convert::{date_from_sql, json_map_from_sql, str_from_sql},
    core::{check_table_exist, Column, ColumnType, Table},
    util::{self, select_all_rows, select_one_row},
};
use qr_model::{Bucket, BucketStatus, Builtin};
use qr_util::if_present;
use rusqlite::{params, types::Value, Connection, Row};
use serde_json;
use std::str::FromStr;

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

pub fn init_table(conn: &Connection) -> rusqlite::Result<bool> {
    check_table_exist(conn, TABLE_NAME, table_def)
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
                Some(val) => serde_json::to_string(val)
                    .map_err(|_| rusqlite::Error::InvalidColumnName("payload".to_string()))?,
                None => "{}".to_string(),
            },
            bucket.builtin,
            bucket.builtin_ref_id,
        ])
        .map(|_| conn.last_insert_rowid())
}

pub fn select_bucket(conn: &Connection, id: i64) -> rusqlite::Result<Option<Bucket>> {
    let sql = format!("SELECT * FROM {} WHERE id = :id limit 1", TABLE_NAME);
    util::select_one_row(conn, &sql, &[(":id", &id.to_string())], map_row)
}

fn map_row(row: &Row<'_>) -> rusqlite::Result<Bucket> {
    let builtin = str_from_sql(row.get("builtin")).and_then(|val| Builtin::from_str(&val).ok());
    let status = str_from_sql(row.get("status"))
        .and_then(|val: String| BucketStatus::from_str(&val).ok())
        .unwrap_or_default();
    Ok(Bucket {
        id: row.get("id")?,
        no: row.get("no")?,
        name: row.get("name")?,
        builtin,
        builtin_ref_id: row.get("builtin_ref_id")?,
        status,
        desc: row.get("desc")?,
        url: row.get("url")?,
        tag: None,
        created: date_from_sql(row.get("create_time")),
        last_modified: date_from_sql(row.get("modify_time")),
        payload: json_map_from_sql(row.get("payload")),
    })
}

pub fn select_bucket_by_builtin(
    conn: &Connection,
    builtin: &Builtin,
    builtin_ref_id: Option<String>,
) -> rusqlite::Result<Option<Bucket>> {
    let b_str = builtin.as_ref();
    let (sql, params) = match builtin_ref_id {
        Some(ref_id) => (
            format!(
                "SELECT * FROM {} WHERE builtin = ?1 and builtin_ref_id = ?2 LIMIT 1",
                TABLE_NAME
            ),
            params![b_str, &ref_id.clone()],
        ),
        None => (
            format!(
                "SELECT * FROM {} WHERE builtin = ?1 and builtin_ref_id IS NULL LIMIT 1",
                TABLE_NAME
            ),
            params![b_str],
        ),
    };
    select_one_row(conn, &sql, params, map_row)
}

pub struct BucketQuery {
    pub builtin: Option<Builtin>,
    pub ref_id: Option<String>,
}

impl BucketQuery {
    pub fn default() -> Self {
        return BucketQuery {
            builtin: None,
            ref_id: None,
        };
    }
}

pub fn select_all_buckets(conn: &Connection, query: BucketQuery) -> rusqlite::Result<Vec<Bucket>> {
    let mut clauses: Vec<String> = vec![String::from("true")];
    let mut p: Vec<Value> = Vec::new();
    let mut p_idx = 1;

    let BucketQuery { builtin, ref_id } = query;
    if_present(builtin, |v| {
        clauses.push(format!("builtin = ?{}", p_idx));
        p_idx += 1;
        p.push(Value::Text(v.to_string()));
    });
    if_present(ref_id, |v| {
        clauses.push(format!("builtin_ref_id = ?{}", p_idx));
        p_idx += 1;
        p.push(Value::Text(v));
    });

    select_all_rows(conn, TABLE_NAME, clauses, p, map_row)
}

pub fn update_bucket(conn: &Connection, bucket: &Bucket) -> rusqlite::Result<()> {
    let sql = format!(
        "UPDATE {} SET modify_time = CURRENT_TIMESTAMP, name = ?, desc = ?, payload = ? WHERE id = ?",
        TABLE_NAME
    );
    let mut stmt = conn.prepare(&sql)?;
    let payload_json = match &bucket.payload {
        Some(val) => serde_json::to_string(val)
            .map_err(|_| rusqlite::Error::InvalidColumnName("payload".to_string()))?,
        None => "{}".to_string(),
    };
    stmt.execute(params![bucket.name, bucket.desc, payload_json, bucket.id])
        .map(|_| ())
}

pub fn delete_bucket(conn: &Connection, bucket_id: i64) -> rusqlite::Result<()> {
    let sql = format!("DELETE FROM {} where id = ?1", TABLE_NAME);
    conn.execute(&sql, [bucket_id]).map(|_| ())
}

pub fn select_all_ids_by_builtin(
    conn: &Connection,
    builtin: &Builtin,
    ref_id: Option<&str>,
) -> rusqlite::Result<Vec<i64>> {
    let (sql, param) = match ref_id {
        Some(v) => (
            format!(
                "SELECT id from {} WHERE builtin = ?1 and builtin_ref_id = ?2",
                TABLE_NAME
            ),
            params![builtin, String::from(v)],
        ),
        None => (
            format!("SELECT id from {} WHERE builtin = ?1", TABLE_NAME),
            params![builtin],
        ),
    };
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(param)?;
    let mut res: Vec<i64> = Vec::new();
    while let Some(row) = rows.next()? {
        let id = row.get(0)?;
        res.push(id);
    }
    Ok(res)
}

#[test]
fn test() {
    use serde_json::{Map, Number, Value};

    let conn = Connection::open_in_memory().unwrap();
    let _ = init_table(&conn);

    let mut payload = Map::new();
    payload.insert("test".to_string(), Value::String("fff".to_string()));
    payload.insert("a1".to_string(), Value::Number(Number::from(123)));

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
        created: None,
        last_modified: None,
        payload: Some(payload),
    };
    let id = insert_bucket(&conn, &bucket).unwrap();
    assert!(id > 0);

    let mut bucket_opt = select_bucket(&conn, id).unwrap();
    let b = bucket_opt.unwrap();
    assert_eq!(b.id.clone().unwrap(), 1);
    assert_eq!(b.no.clone().unwrap(), 234);
    assert_eq!(b.name, "foobar".to_string());
    assert_eq!(b.desc.clone().unwrap(), "Mock description".to_string());
    assert_eq!(b.url.clone().unwrap(), "https://qr.com".to_string());

    bucket_opt = select_bucket(&conn, 2).unwrap();
    assert!(bucket_opt.is_none());
}
