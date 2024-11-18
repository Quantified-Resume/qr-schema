use crate::service::stat::common::sum;

use super::{query::build_clauses_and_params, AggregationMethod, CommonFilter};
use chrono::{DateTime, Datelike};
use qr_model::Item;
use qr_repo::select_all_items;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Clone, Serialize, Deserialize, Debug)]
pub enum IndicatorCycle {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorRequest {
    filter: CommonFilter,
    aggregation: AggregationMethod,
    average_cycle: Option<IndicatorCycle>,
}

fn count(items: &Vec<Item>, metrics: &str) -> Result<Decimal, String> {
    let mut count = 0;
    for item in items {
        match item.metrics.get(metrics) {
            Some(_) => count += 1,
            None => {}
        }
    }
    Ok(Decimal::new(count, 0))
}

fn count_cycle(
    items: &Vec<Item>,
    cycle: &Option<IndicatorCycle>,
    filter: &CommonFilter,
) -> Result<i64, String> {
    let count = match cycle {
        Some(v) => {
            let ts_offset = filter.utc_offset * 60 * 1000;
            let ts_start = filter
                .ts_start
                .ok_or("Ts start is required for average with cycle".to_string())?;
            let start = DateTime::from_timestamp_millis(ts_start)
                .ok_or("Start time is invalid".to_string())?;
            let ts_end = filter
                .ts_end
                .ok_or("Ts end is required for average with cycle".to_string())?;
            let end =
                DateTime::from_timestamp_millis(ts_end).ok_or("End time is invalid".to_string())?;
            let y_diff = (end.year() - start.year()) as i64;
            let m_diff = end.month() as i64 - start.month() as i64;

            match v {
                IndicatorCycle::Hourly => {
                    let h_start = (ts_start - ts_offset) / 60 / 60 / 1000;
                    let h_end = (ts_end - ts_offset) / 60 / 60 / 1000;
                    h_end - h_start + 1
                }
                IndicatorCycle::Daily => {
                    let d_start = (ts_start - ts_offset) / 24 / 60 / 60 / 1000;
                    let d_end = (ts_end - ts_offset) / 24 / 60 / 60 / 1000;
                    d_end - d_start + 1
                }
                IndicatorCycle::Weekly => {
                    // Not supported yet
                    Err("Not supported weekly yet")?
                }
                IndicatorCycle::Monthly => y_diff * 12 + m_diff + 1,
                IndicatorCycle::Yearly => y_diff + 1,
            }
        }
        None => items.len() as i64,
    };
    Ok(count)
}

pub fn query_indicator(conn: &Connection, req: &IndicatorRequest) -> Result<Decimal, String> {
    let metrics = req
        .filter
        .metrics
        .clone()
        .and_then(|v| v.get(0).map(|s| s.to_string()))
        .ok_or("Metrics is required".to_string())?;
    let (clauses, params) = build_clauses_and_params(conn, &req.filter)?;
    let items = select_all_items(conn, clauses, params).map_err(|e| {
        log::error!("Failed to query items: {:?}", e);
        "Failed to query items".to_string()
    })?;
    log::info!("{} items found for indicator", items.len());
    let aggregation = &req.aggregation;
    let cycle = &req.average_cycle;
    match aggregation {
        AggregationMethod::Sum => sum(&items, &metrics),
        AggregationMethod::Average => {
            let sum = sum(&items, &metrics)?;
            let count = count_cycle(&items, cycle, &req.filter)?;
            let average = sum
                .checked_div(Decimal::new(count as i64, 0))
                .unwrap_or(Decimal::new(0, 0));
            Ok(average)
        }
        AggregationMethod::Count => count(&items, &metrics),
    }
}

#[test]
fn test_count_cycle() {
    let filter = CommonFilter {
        bucket: crate::service::BucketKey {
            id: None,
            builtin: None,
            builtin_ref_id: None,
        },
        utc_offset: -480,
        ts_start: Some(1731427200000),
        ts_end: Some(1731489428526),
        payload: None,
        metrics: None,
    };
    let count = count_cycle(&vec![], &Some(IndicatorCycle::Daily), &filter);
    assert_eq!(1, count.unwrap());
}
