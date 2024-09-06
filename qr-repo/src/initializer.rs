use rusqlite::Connection;

use crate::{bucket, item, sequence};

pub fn init_tables(conn: &Connection) {
    let success = bucket::init_table(conn)
        .and(sequence::init_table(conn))
        .and(item::init_table(conn));
    match success {
        Ok(_) => {}
        Err(e) => panic!("Failed to init tables: {}", e),
    }
}
