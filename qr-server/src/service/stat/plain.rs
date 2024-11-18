use std::vec;

use qr_repo::select_all_items;
use rocket::serde;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::service::stat::query::build_clauses_and_params;

use super::CommonFilter;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlainItem {
    id: i64,
    timestamp: i64,
    name: Option<String>,
    ref_id: String,
    payload: Option<serde_json::Map<String, serde_json::Value>>,
    metrics: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlainResult {
    query: CommonFilter,
    version: i64,
    items: Vec<PlainItem>,
}

impl PlainResult {
    /// Version 1
    fn v1(req: &CommonFilter, items: &Vec<PlainItem>) -> Self {
        PlainResult {
            version: 1,
            query: req.clone(),
            items: items.clone(),
        }
    }
}

/// Query items
pub fn query_plain(conn: &Connection, filter: &CommonFilter) -> Result<PlainResult, String> {
    log::info!("query_plain: req={:?}", filter);
    let (clauses, params) = build_clauses_and_params(conn, filter)?;
    let mut items = select_all_items(conn, clauses, params).map_err(|e| {
        log::error!("Failed to query items: err={}", e);
        "Failed to query items".to_string()
    })?;
    items.sort_by_key(|v| v.timestamp);
    let plain_items = cvt_item(&items, &filter.metrics);
    Ok(PlainResult::v1(filter, &plain_items))
}

fn cvt_item(items: &Vec<qr_model::Item>, target_metrics: &Option<Vec<String>>) -> Vec<PlainItem> {
    let valid_metrics = target_metrics.clone().take_if(|v| !v.is_empty());
    items
        .iter()
        .map(|v| PlainItem {
            id: v.id.unwrap_or(0),
            name: v.name.clone(),
            ref_id: v.ref_id.clone(),
            timestamp: v.timestamp,
            payload: v.payload.clone(),
            metrics: extract_metrics(&v.metrics, &valid_metrics),
        })
        .collect()
}

fn extract_metrics(
    value: &serde_json::Map<String, serde_json::Value>,
    target_opt: &Option<Vec<String>>,
) -> serde_json::Map<String, serde_json::Value> {
    let target = target_opt.clone().unwrap_or(vec![]);
    if target.is_empty() {
        return value.clone();
    }
    let iter = value.iter().filter(|(k, _v)| target.contains(&k));
    serde_json::Map::from_iter(iter.map(|(k, v)| (k.to_owned(), v.to_owned())))
}
