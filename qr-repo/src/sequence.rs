use chrono::format::format;
use rusqlite::{params, Connection};

use crate::{
    core::{check_table_exist, Column, ColumnType, Table},
    util::{self, select_one_row},
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
    // TODO
    let sql = format!("INSERT INTO {} VALUES ()", TABLE_NAME);
}

pub fn next_seq(conn: &Connection, sequence: Sequence) -> i64 {
    let conf = seq_config(sequence);
    let sql = format!("SELECT value FROM {} WHERE key = :key", TABLE_NAME);
    let exist: Option<i64> =
        util::select_one_row(conn, &sql, &[(":key", &conf.key)], |row| row.get(0));
    match exist {
        Some(val) => {}
        None => {
            init_seq(conn, &conf);
            conf.initial
        }
    }
}
