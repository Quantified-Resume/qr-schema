use std::vec;

use crate::err::err;

use super::BucketKey;
use itertools::Itertools;
use qr_repo::{select_all_ids_by_builtin, select_bucket, QueryCommand};
use qr_util::if_present;
use rusqlite::{types::Value, Connection};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct QueryRequest {
    pub bucket: BucketKey,
    pub ts_start: Option<i64>,
    pub ts_end: Option<i64>,
}

pub fn stat_profile(conn: &Connection, req: QueryRequest) -> Result<i64, String> {
    let bucket_ids = match check_bucket(conn, &req.bucket) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let cmd = match built_cmd(bucket_ids, &req) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    Ok(1)
}

fn built_cmd(bucket_ids: Vec<i64>, req: &QueryRequest) -> Result<QueryCommand, String> {
    let mut clause: Vec<String> = Vec::new();
    let mut p: Vec<Value> = Vec::new();
    let mut p_idx = 1;
    // 1. bucket_ids
    if bucket_ids.is_empty() {
        return Ok(QueryCommand::and_false());
    }
    let b_sql = bucket_ids
        .iter()
        .map(|bid| {
            p.push(Value::Integer(*bid));
            let placeholder = format!("?{}", p_idx);
            p_idx += 1;
            placeholder
        })
        .join(", ");
    clause.push(format!("bucket_id in ({})", b_sql));
    // 2. time
    if_present(req.ts_start, |ts| {
        clause.push(format!("ts >= ?{}", p_idx));
        p_idx += 1;
        p.push(Value::Integer(ts));
    });
    if_present(req.ts_end, |ts| {
        clause.push(format!("ts < ?{}", p_idx));
        p_idx += 1;
        p.push(Value::Integer(ts));
    });
    // 3. payload

    // 4. metrics

    Ok(QueryCommand {
        sql: clause.join(" and "),
        params: p,
        columns: vec![],
        group_by: None,
    })
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

#[test]
pub fn test_cmg() {
    let req = QueryRequest {
        bucket: BucketKey {
            builtin_ref_id: None,
            id: None,
            builtin: None,
        },
        ts_start: Some(1000),
        ts_end: Some(2000),
    };

    match built_cmd(vec![10, 20], &req) {
        Ok(cmd) => {
            assert_eq!(cmd.sql, "bucket_id in (?1, ?2) and ts >= ?3 and ts < ?4");
            assert_eq!(cmd.params.len(), 4);
        }
        Err(_) => {}
    }
}
