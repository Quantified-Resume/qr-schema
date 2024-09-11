use std::vec;

use crate::err::err;

use super::BucketKey;
use qr_repo::{select_all_ids_by_builtin, select_bucket};
use rusqlite::Connection;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProfileRequest {
    pub bucket: BucketKey,
    pub metrics: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub fn stat_profile(conn: &Connection, req: ProfileRequest) -> Result<i64, String> {
    let bucket_ids = match check_bucket(conn, &req.bucket) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    Ok(1)
}

fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Vec<i64>, String> {
    let bucket_ids_res = match key.id {
        Some(id) => match select_bucket(conn, id) {
            Some(_) => return Ok(vec![id]),
            None => return Err("Bucket not found".to_string()),
        },
        None => match &key.builtin {
            Some(v) => select_all_ids_by_builtin(conn, v, key.builtin_ref_id.as_deref()),
            None => return Err("Invalid bucket key".to_string()),
        },
    };
    match bucket_ids_res {
        Ok(ids) => Ok(ids),
        Err(e) => err(e, "Failed to find buckets"),
    }
}
