use rusqlite::{ffi::Error, params, Connection};

use crate::{
    core::{check_table_exist, Column, ColumnType, Table},
    util::select_one_row,
};

const TABLE_NAME: &str = "sequence";

fn table_def() -> Table {
    Table {
        name: String::from(TABLE_NAME),
        columns: vec![
            Column::required("key", ColumnType::Text),
            Column::required("value", ColumnType::Integer),
            Column::with_default("step", ColumnType::Integer, "1"),
        ],
    }
}

pub fn init_table(conn: &Connection) {
    check_table_exist(conn, TABLE_NAME, table_def);
}

pub enum Sequence {
    Bucket,
}

struct SequenceConf {
    initial: i64,
    key: String,
    step: i64,
}

impl SequenceConf {
    fn new(initial: i64, key: &str, step: i64) -> Self {
        SequenceConf {
            initial,
            key: key.to_string(),
            step,
        }
    }
}

fn seq_config(seq: Sequence) -> SequenceConf {
    match seq {
        Sequence::Bucket => SequenceConf::new(10000, "bucket", 3),
    }
}

fn init_seq(conn: &Connection, conf: &SequenceConf) {
    let sql = format!(
        "INSERT INTO {} (key, value, step) VALUES (?, ?, ?)",
        TABLE_NAME
    );
    conn.execute(&sql, params![conf.key, conf.initial, conf.step])
        .expect("Failed to init sequence");
}

pub fn next_seq(conn: &Connection, sequence: Sequence) -> Result<i64, Error> {
    let conf = seq_config(sequence);

    let sql: String = format!("SELECT value, step FROM {} WHERE key = :key", TABLE_NAME);
    let exist: Option<(i64, i64)> = select_one_row(conn, &sql, &[(":key", &conf.key)], |row| {
        let value_cell: Result<i64, rusqlite::Error> = row.get(0);
        let step_cell: Result<i64, rusqlite::Error> = row.get(1);
        value_cell.and_then(|value| step_cell.map(|step| (value, step)))
    });
    match exist {
        Some((val, step)) => {
            let sql = format!(
                "UPDATE {} SET value = value + step where value = ? and key = ?",
                TABLE_NAME
            );
            conn.prepare(&sql)
                .and_then(|mut s| s.execute(params![val, conf.key]))
                .and_then(|_| Ok(val + step))
                .map_err(|_| Error::new(0i32))
        }
        None => {
            init_seq(conn, &conf);
            Ok(conf.initial)
        }
    }
}
