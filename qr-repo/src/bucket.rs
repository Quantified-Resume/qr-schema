use qr_model::Bucket;
use rusqlite::{params, Connection};
use serde_json;

use crate::{
    convert::{date_from_sql, json_map_from_sql},
    core::{check_table_exist, Column, ColumnType, Table},
    util,
};

const TABLE_NAME: &str = "bucket";

fn table_def() -> Table {
    return Table {
        name: TABLE_NAME.to_string(),
        columns: vec![
            Column::required("no", ColumnType::Text),
            Column::required("name", ColumnType::Text),
            Column::nullable("desc", ColumnType::Text),
            Column::nullable("url", ColumnType::Text), // Reserved field
            Column::nullable("payload", ColumnType::Json),
            Column::with_default("built_in_flag", ColumnType::Bool, "false"), // Reserved field
        ],
    };
}

pub fn init_table(conn: &Connection) {
    check_table_exist(conn, TABLE_NAME, table_def);
}

pub fn insert_bucket(conn: &Connection, bucket: &Bucket) -> i64 {
    let sql = format!(
        "INSERT INTO {} (no, name, desc, url, payload) VALUES (?1, ?2, ?3, ?4, ?5);",
        TABLE_NAME
    );
    let mut statement = conn.prepare(&sql).unwrap();
    let _ = statement
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
        .expect("Insert bucket failure");
    conn.last_insert_rowid()
}

pub fn select_bucket_by_id(conn: &Connection, id: i64) -> Option<Bucket> {
    let sql = format!(
        "SELECT id, no, name, desc, url, payload, create_time, modify_time from {} WHERE id = :id limit 1",
        TABLE_NAME
    );
    util::select_one_row(conn, &sql, &[(":id", &id.to_string())], |row| {
        Ok(Bucket {
            id: row.get_unwrap(0),
            no: row.get_unwrap(1),
            name: row.get_unwrap(2),
            desc: row.get_unwrap(3),
            url: row.get_unwrap(4),
            tag: None,
            created: date_from_sql(row.get(6)).expect("Failed to query create_time"),
            last_modified: date_from_sql(row.get(7)).expect("Failed to query modify_time"),
            payload: json_map_from_sql(row.get(5)),
        })
    })
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
        no: Some("234".to_string()),
        name: "foobar".to_string(),
        desc: Some("Mock description".to_string()),
        url: Some("https://qr.com".to_string()),
        tag: None,
        created: DateTime::default(),
        last_modified: DateTime::default(),
        payload: Some(payload),
    };
    let id = insert_bucket(&conn, &bucket);
    assert!(id > 0);
    println!("Bucket inserted: id={}", id);

    let mut bucket_res = select_bucket_by_id(&conn, id);
    let b = bucket_res.unwrap();
    assert_eq!(b.id.clone().unwrap(), 1);
    assert_eq!(b.no.clone().unwrap(), "234".to_string());
    assert_eq!(b.name, "foobar".to_string());
    assert_eq!(b.desc.clone().unwrap(), "Mock description".to_string());
    assert_eq!(b.url.clone().unwrap(), "https://qr.com".to_string());
    println!("{}", b);

    bucket_res = select_bucket_by_id(&conn, 2);
    assert!(bucket_res.is_none());
}
