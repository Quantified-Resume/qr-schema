use std::vec;

use crate::err::err;

use super::BucketKey;
use itertools::Itertools;
use jsonpath::Selector;
use qr_repo::{select_all_ids_by_builtin, select_all_items, select_bucket};
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
    pub val: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub bucket: BucketKey,
    pub ts_start: Option<i64>,
    pub ts_end: Option<i64>,
    pub payload_filter: Option<Vec<PayloadFilter>>,
}

pub fn query_stat(conn: &Connection, req: QueryRequest) -> Result<i64, String> {
    log::info!("query_stat: req={:?}", req);
    let bucket_ids = match check_bucket(conn, &req.bucket) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let (clauses, params) = match built_clauses_and_params(bucket_ids, &req) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let items = match select_all_items(conn, clauses, params) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to query items: err={}", e);
            return Err("Failed to query items".to_string());
        }
    };
    Ok(1)
}

fn built_clauses_and_params(
    bucket_ids: Vec<i64>,
    req: &QueryRequest,
) -> Result<(Vec<String>, Vec<Value>), String> {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    let mut p_idx = 1;
    // 1. bucket_ids
    if bucket_ids.is_empty() {
        return Ok((vec![String::from("false")], vec![]));
    }
    let b_sql = bucket_ids
        .iter()
        .map(|bid| {
            params.push(Value::Integer(*bid));
            let placeholder = format!("?{}", p_idx);
            p_idx += 1;
            placeholder
        })
        .join(", ");
    clauses.push(format!("bucket_id in ({})", b_sql));
    // 2. time
    if_present(req.ts_start, |ts| {
        clauses.push(format!("timestamp >= ?{}", p_idx));
        p_idx += 1;
        params.push(Value::Integer(ts));
    });
    if_present(req.ts_end, |ts| {
        clauses.push(format!("timestamp < ?{}", p_idx));
        p_idx += 1;
        params.push(Value::Integer(ts));
    });
    // 3. payload
    let pf_res = match &req.payload_filter {
        Some(v) => handle_payload_filter(v, &mut clauses, &mut params, p_idx),
        None => Ok(p_idx),
    };
    let _ = match pf_res {
        Ok(new_idx) => new_idx,
        Err(e) => return Err(e),
    };
    Ok((clauses, params))
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
                let param_val = match val {
                    Some(v) => v,
                    None => return Err("Value can't be null".to_string()),
                };
                if param_val.is_null() || param_val.is_object() || param_val.is_array() {
                    return Err("Invalid value type for Eq or Neq".to_string());
                }
                clause.push(format!(
                    "json_extract(payload, '{}') {} ?{}",
                    path,
                    op.to_sql(),
                    idx
                ));
                p.push(json_val_to_sql_val(&param_val));
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
