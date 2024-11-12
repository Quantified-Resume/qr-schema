use super::{query::build_clauses_and_params, CommonFilter};
use qr_model::Item;
use qr_repo::select_all_items;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{any::Any, str::FromStr};
use strum::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Clone, Serialize, Deserialize, Debug)]
pub enum IndicatorAggregation {
    Sum,
    Average,
    Count,
}

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
    aggregation: IndicatorAggregation,
    average_cycle: Option<IndicatorCycle>,
}

fn cvt_json_2_decimal(
    val: Option<&serde_json::Value>,
    default_zero: bool,
) -> Result<Decimal, String> {
    match val {
        Some(v) => {
            if v.is_string() || v.is_number() {
                let val_str = v.to_string();
                return Decimal::from_str(&val_str).map_err(|e| {
                    log::error!("Invalid decimal value: {}, e={:?}", val_str, e);
                    "Invalid decimal value".to_string()
                });
            }
            Err(format!(
                "Unsupported value type: type={:?}, value={:?}",
                v.type_id(),
                v
            ))
        }
        None => match default_zero {
            true => Ok(Decimal::new(0, 0)),
            false => Err("Value is null".to_string()),
        },
    }
}

fn sum(items: &Vec<Item>, metrics: &str) -> Result<Decimal, String> {
    let mut sum = Decimal::new(0, 0);
    for item in items {
        let v = cvt_json_2_decimal(item.metrics.get(metrics), true)?;
        sum = sum.saturating_add(v);
    }
    Ok(sum)
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
    let aggregation = req.aggregation.clone();
    match aggregation {
        IndicatorAggregation::Sum => sum(&items, &metrics),
        IndicatorAggregation::Average => todo!(),
        IndicatorAggregation::Count => count(&items, &metrics),
    }
}
