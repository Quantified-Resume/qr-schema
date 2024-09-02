use std::str::FromStr;

use qr_model::{Bucket, Builtin};
use rusqlite::{ffi::Error, params, Connection};
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
            Column::required("no", ColumnType::Text),
            Column::required("name", ColumnType::Text),
            Column::nullable("builtin", ColumnType::Text),
            Column::nullable("builtin_ref_id", ColumnType::Text), // Ref ID of builtin channel
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

pub fn insert_bucket(conn: &Connection, bucket: &Bucket) -> Result<i64, Error> {
    let sql = format!(
        "INSERT INTO {} (no, name, desc, url, payload) VALUES (?1, ?2, ?3, ?4, ?5);",
        TABLE_NAME
    );
    let mut statement = conn.prepare(&sql).unwrap();
    statement
        .execute(params![
            bucket.no,
            bucket.name,
            bucket.desc,
            bucket.url,
            match &bucket.payload {
                Some(val) => serde_json::to_string(val).unwrap(),
                None => "{}".to_string(),
            },
        ])
        .and_then(|_| Ok(conn.last_insert_rowid()))
        .map_err(|_| Error::new(-1i32))
}

const ALL_COLUMNS: &str =
    "id, no, name, builtin, builtin_ref_id, desc, url, payload, create_time, modify_time";

pub fn select_bucket_by_id(conn: &Connection, id: i64) -> Option<Bucket> {
    let sql = format!(
        "SELECT {} FROM {} WHERE id = :id limit 1",
        ALL_COLUMNS, TABLE_NAME
    );
    util::select_one_row(conn, &sql, &[(":id", &id.to_string())], |row| {
        let builtin = str_from_sql(row.get("builtin")).and_then(|val| Builtin::from_str(&val).ok());
        Ok(Bucket {
            id: row.get_unwrap("id"),
            no: row.get_unwrap("no"),
            name: row.get_unwrap("name"),
            builtin,
            builtin_ref_id: row.get("builtin_ref_id").ok(),
            desc: row.get_unwrap("desc"),
            url: row.get_unwrap("url"),
            tag: None,
            created: Some(
                date_from_sql(row.get("create_time")).expect("Failed to query create_time"),
            ),
            last_modified: Some(
                date_from_sql(row.get("modify_time")).expect("Failed to query modify_time"),
            ),
            payload: json_map_from_sql(row.get("payload")),
        })
    })
}

pub fn select_bucket_by_builtin(
    conn: &Connection,
    builtin: Builtin,
    builtin_ref_id: Option<String>,
) -> Option<Bucket> {
    let mut sql = format!(
        "SELECT {} FROM {} WHERE builtin = :b",
        ALL_COLUMNS, TABLE_NAME
    );
    if builtin_ref_id.is_some() {
        sql.push_str(" AND builtin_ref_id = :b_id");
    };
    sql.push_str(" LIMIT 1");
    select_one_row(conn, &sql, [(":b", builtin.)], f)
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

    let mut bucket_res = select_bucket_by_id(&conn, id);
    let b = bucket_res.unwrap();
    assert_eq!(b.id.clone().unwrap(), 1);
    assert_eq!(b.no.clone().unwrap(), 234);
    assert_eq!(b.name, "foobar".to_string());
    assert_eq!(b.desc.clone().unwrap(), "Mock description".to_string());
    assert_eq!(b.url.clone().unwrap(), "https://qr.com".to_string());
    println!("{}", b);

    bucket_res = select_bucket_by_id(&conn, 2);
    assert!(bucket_res.is_none());
}
