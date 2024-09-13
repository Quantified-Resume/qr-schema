use std::vec;

use crate::err::err;

use super::BucketKey;
use itertools::Itertools;
use jsonpath::Selector;
use qr_repo::{query_items, select_all_ids_by_builtin, select_bucket, QueryCommand};
use qr_util::if_present;
use rusqlite::{types::Value, Connection};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum FilterOp {
    Eq,
    Neq,
    IsNul,
    NotNul,
}

impl FilterOp {
    fn to_sql(&self) -> String {
        match self {
            FilterOp::Eq => "=".to_string(),
            FilterOp::Neq => "!=".to_string(),
            FilterOp::IsNul => "IS NULL".to_string(),
            FilterOp::NotNul => "IS NOT NULL".to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PayloadFilter {
    pub path: String,
    pub op: FilterOp,
    pub val: serde_json::Value,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub bucket: BucketKey,
    pub ts_start: Option<i64>,
    pub ts_end: Option<i64>,
    pub payload_filter: Option<Vec<PayloadFilter>>,
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
    let items = match query_items(conn, &cmd) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to query items: err={}", e);
            return Err("Failed to query items".to_string());
        }
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
    let pf_res = match &req.payload_filter {
        Some(v) => handle_payload_filter(v, &mut clause, &mut p, p_idx),
        None => Ok(p_idx),
    };
    match pf_res {
        Ok(new_idx) => p_idx = new_idx,
        Err(e) => return Err(e),
    };
    // 4. metrics
    Ok(QueryCommand {
        clause: clause.join(" and "),
        params: p,
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

fn handle_payload_filter(
    pf: &Vec<PayloadFilter>,
    clause: &mut Vec<String>,
    p: &mut Vec<Value>,
    idx_start: i32,
) -> Result<i32, String> {
    let mut idx = idx_start;
    for f in pf {
        let PayloadFilter { path, op, val } = f;
        match Selector::new(&path) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to parse json path: {:?}", e);
                return Err(format!("Invalid json path: {}", path));
            }
        };
        match op {
            FilterOp::Eq | FilterOp::Neq => {
                if val.is_null() || val.is_object() || val.is_array() {
                    return Err("Invalid value type for Eq".to_string());
                }
                clause.push(format!(
                    "json_extract(payload, '{}') {} ?{}",
                    path,
                    op.to_sql(),
                    idx
                ));
                p.push(json_val_to_sql_val(&val));
                idx += 1;
            }
            FilterOp::IsNul | FilterOp::NotNul => {
                clause.push(format!("json_extract(payload, '{}') {}", path, op.to_sql()))
            }
        };
    }
    Ok(idx)
}

fn json_val_to_sql_val(json: &serde_json::Value) -> Value {
    if json.is_null() {
        return Value::Null;
    } else if json.is_boolean() {
        let v = match json.as_bool() {
            Some(v) => match v {
                true => 1,
                false => 0,
            },
            None => 0,
        };
        return Value::Integer(v);
    } else if json.is_i64() {
        let v = match json.as_i64() {
            Some(v) => v,
            None => 0,
        };
        return Value::Integer(v);
    } else if json.is_u64() {
        let v = match json.as_u64() {
            Some(v) => v,
            None => 0,
        };
        return Value::Integer(v as i64);
    } else {
        return Value::Text(json.to_string());
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
        payload_filter: Some(vec![PayloadFilter {
            path: "$.a.val".to_string(),
            op: FilterOp::Eq,
            val: serde_json::Value::String("test".to_string()),
        }]),
    };

    match built_cmd(vec![10, 20], &req) {
        Ok(cmd) => {
            println!("{:?}", cmd);
            assert_eq!(cmd.sql, "bucket_id in (?1, ?2) and ts >= ?3 and ts < ?4 and json_extract(payload, '$.a.val') = ?5");
            assert_eq!(cmd.params.len(), 5);
        }
        Err(_) => {}
    }
}
