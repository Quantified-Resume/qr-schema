use qr_repo::select_all_items;
use rocket::serde;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{AsRefStr, EnumString};

use crate::service::stat::{convert::json_val_to_decimal, query::built_clauses_and_params};

use super::CommonFilter;

#[derive(EnumString, AsRefStr, Clone, Serialize, Deserialize, Debug)]
pub enum GroupBy {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Bucket,
    Builtin,
}

impl GroupBy {
    pub fn is_timing_key(self) -> bool {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupRequest {
    filter: CommonFilter,
    group: Vec<GroupBy>,
    // The utc offset, used for calculating time key
    utc_offset: Option<i64>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupMetrics {
    sum: Decimal,
    count: i64,
    max: Decimal,
    min: Decimal,
    average: Decimal,
}

impl GroupMetrics {
    pub fn new() -> Self {
        GroupMetrics {
            sum: Decimal::ZERO,
            count: 0,
            max: Decimal::ZERO,
            min: Decimal::ZERO,
            average: Decimal::ZERO,
        }
    }

    pub fn reduce(&mut self, val: Decimal) {
        self.count += 1;
        todo!()
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupItem {
    group_key: Vec<String>,
    metrics: GroupMetrics,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupResult {
    version: i64,
    metrics: HashMap<String, Vec<GroupItem>>,
}

impl GroupResult {
    pub fn v1(metrics: HashMap<String, Vec<GroupItem>>) -> Self {
        GroupResult {
            version: 1,
            metrics,
        }
    }
}

/// Query items
pub fn query_group(conn: &Connection, request: &GroupRequest) -> Result<GroupResult, String> {
    log::info!("query_group: req={:?}", request);
    let filter = &request.filter;
    let (clauses, params) = built_clauses_and_params(conn, filter)?;
    let mut items = select_all_items(conn, clauses, params).map_err(|e| {
        log::error!("Failed to query items: err={}", e);
        "Failed to query items".to_string()
    })?;
    items.sort_by_key(|v| v.timestamp);
    let group_items = group_by(&items, &request.group, &filter.metrics, request.utc_offset)?;
    Ok(GroupResult::v1(group_items))
}

fn group_by(
    items: &Vec<qr_model::Item>,
    group: &Vec<GroupBy>,
    target_metrics: &Option<Vec<String>>,
    utc_offset: Option<i64>,
) -> Result<HashMap<String, Vec<GroupItem>>, String> {
    let timing_count = group
        .iter()
        .filter(|g| <GroupBy as Clone>::clone(&g).is_timing_key())
        .count();
    let mut offset = 0;
    if timing_count > 1 {
        return Err(format!(
            "No more one timing key, but {} found",
            timing_count
        ));
    } else if timing_count == 1 {
        offset = utc_offset.ok_or("The UTC offset is required for timing key".to_string())?;
    }
    let valid_metrics = target_metrics.clone().unwrap_or(vec![]);
    let mut cache: HashMap<String, HashMap<Vec<String>, GroupMetrics>> = HashMap::new();
    for item in items {
        let key = compute_key(&item, &group, offset);
        for (k, val) in item.metrics.clone() {
            if !valid_metrics.is_empty() && !valid_metrics.contains(&k) {
                continue;
            }
            let group_metrics = cache.entry(k).or_insert_with(|| HashMap::new());

            let metrics = group_metrics
                .entry(key.clone())
                .or_insert_with(GroupMetrics::new);
            let dec_val = json_val_to_decimal(&val)?;
            metrics.reduce(dec_val);
        }
    }
    // convert map to vec
    todo!()
}

fn compute_key(item: &qr_model::Item, group: &Vec<GroupBy>, offset: i64) -> Vec<String> {
    todo!()
}
