use crate::{
    core::{check_table_exist, Column, ColumnType, Table},
    util::select_one_row,
};
use rusqlite::{params, Connection};

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

pub fn init_table(conn: &Connection) -> rusqlite::Result<bool> {
    check_table_exist(conn, TABLE_NAME, table_def)
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

fn init_seq(conn: &Connection, conf: &SequenceConf) -> Result<usize, rusqlite::Error> {
    let sql = format!(
        "INSERT INTO {} (key, value, step) VALUES (?, ?, ?)",
        TABLE_NAME
    );
    conn.execute(&sql, params![conf.key, conf.initial, conf.step])
}

pub fn next_seq(conn: &Connection, sequence: Sequence) -> rusqlite::Result<i64> {
    let conf = seq_config(sequence);

    let sql: String = format!("SELECT value, step FROM {} WHERE key = :key", TABLE_NAME);
    let exist = select_one_row(conn, &sql, &[(":key", &conf.key)], |row| {
        let value_cell: Result<i64, rusqlite::Error> = row.get(0);
        let step_cell: Result<i64, rusqlite::Error> = row.get(1);
        value_cell.and_then(|value| step_cell.map(|step| (value, step)))
    })?;
    match exist {
        Some((val, step)) => {
            let sql = format!(
                "UPDATE {} SET value = value + step where value = ? and key = ?",
                TABLE_NAME
            );
            let mut stmt = conn.prepare(&sql)?;
            stmt.execute(params![val, conf.key])?;
            Ok(val + step)
        }
        None => {
            init_seq(conn, &conf)?;
            Ok(conf.initial)
        }
    }
}
