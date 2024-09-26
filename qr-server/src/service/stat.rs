use super::BucketKey;
use itertools::Itertools;
use jsonpath::Selector;
use qr_model::{ChartSeries, Item};
use qr_repo::{select_all_ids_by_builtin, select_all_items, select_bucket};
use qr_util::if_present;
use rusqlite::{types::Value, Connection};
use serde::{Deserialize, Serialize};
use std::vec;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PayloadFilter {
    pub path: String,
    pub op: FilterOp,
    pub val: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartQueryRequest {
    pub bucket: BucketKey,
    pub ts_start: Option<i64>,
    pub ts_end: Option<i64>,
    pub payload_filter: Option<Vec<PayloadFilter>>,
    pub metrics: Option<Vec<String>>,
    pub series: ChartSeries,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChartResult {
    query: ChartQueryRequest,
    version: i64,
    option: serde_json::Map<String, serde_json::Value>,
}

impl ChartResult {
    fn v1(req: &ChartQueryRequest, option: serde_json::Map<String, serde_json::Value>) -> Self {
        ChartResult {
            version: 1,
            query: req.clone(),
            option,
        }
    }
}

pub fn query_chart(conn: &Connection, req: ChartQueryRequest) -> Result<ChartResult, String> {
    log::info!("query_chart: req={:?}", req);
    let ChartQueryRequest {
        bucket,
        series,
        metrics,
        ..
    } = req.clone();
    let bucket_ids = check_bucket(conn, &bucket)?;
    let (clauses, params) = built_clauses_and_params(bucket_ids, &req)?;
    let items = select_all_items(conn, clauses, params).map_err(|e| {
        log::error!("Failed to query items: err={}", e);
        "Failed to query items".to_string()
    })?;
    let option = generate_opt(&series, &metrics, items);
    let result = ChartResult::v1(&req, option);
    // TODO
    Ok(result)
}

fn built_clauses_and_params(
    bucket_ids: Vec<i64>,
    req: &ChartQueryRequest,
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
    let _new_p_idx = match &req.payload_filter {
        Some(v) => handle_payload_filter(v, &mut clauses, &mut params, p_idx),
        None => Ok(p_idx),
    }?;
    Ok((clauses, params))
}

fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Vec<i64>, String> {
    match key.id {
        Some(id) => select_bucket(conn, id)
            .map_err(|e| {
                log::error!("Errored to find bucket: id={}, e={}", id, e);
                "Errored to find bucket".to_string()
            })?
            .ok_or("Bucket not found".to_string())
            .map(|_| vec![id]),
        None => {
            let b = &key
                .builtin
                .clone()
                .ok_or("Invalid bucket key".to_string())?;
            select_all_ids_by_builtin(conn, b, key.builtin_ref_id.as_deref()).map_err(|e| {
                log::error!("Failed to select ids by builtin: e={}, builtin={:?}", e, b);
                "Failed to select all ids by builtin".to_string()
            })
        }
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
        Selector::new(&path).map_err(|e| {
            log::error!("Failed to parse json path: {:?}", e);
            format!("Invalid json path: {}", path)
        })?;
        match op {
            FilterOp::Eq | FilterOp::Neq => {
                let param_val = val.clone().ok_or("Value can't be null".to_string())?;
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
        let v = json
            .as_bool()
            .map(|v| match v {
                true => 1,
                false => 0,
            })
            .unwrap_or(0);
        return Value::Integer(v);
    } else if json.is_i64() {
        let v = json.as_i64().unwrap_or(0);
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

fn generate_opt(
    series: &ChartSeries,
    metrics: &Option<Vec<String>>,
    items: Vec<Item>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    match series {
        ChartSeries::CalendarHeat => {}
        ChartSeries::Line => {}
    };
    map
}
