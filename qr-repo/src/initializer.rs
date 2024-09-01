use rusqlite::Connection;

use crate::{bucket, sequence};

pub fn init_tables(conn: &Connection) {
    bucket::init_table(conn);
    sequence::init_table(conn);
}
