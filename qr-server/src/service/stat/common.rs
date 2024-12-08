use std::{any::Any, str::FromStr};

use chrono::{DateTime, FixedOffset};
use indexmap::IndexMap;
use qr_model::Item;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::CommonFilter;

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

pub fn sum(items: &Vec<Item>, metrics: &str) -> Result<Decimal, String> {
    let mut sum = Decimal::new(0, 0);
    for item in items {
        let v = cvt_json_2_decimal(item.metrics.get(metrics), true)?;
        sum = sum.saturating_add(v);
    }
    Ok(sum)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KeyValueType {
    Date,
    Metrics,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeriesQuery {
    pub metrics: String,
    // todo
    // meta: MetaSeries,
}

// Group items by dates according to filter param
pub fn group_by_dates(
    filter: &CommonFilter,
    items: &Vec<Item>,
) -> Result<IndexMap<String, Vec<Item>>, String> {
    let offset_secs = (filter.utc_offset * 60) as i32;
    let tz = FixedOffset::west_opt(offset_secs).ok_or("Invalid offset value".to_string())?;

    let mut map: IndexMap<String, Vec<Item>> = IndexMap::new();
    for item in items {
        let utc = DateTime::from_timestamp_millis(item.timestamp)
            .ok_or(format!("Invalid time stamp of item: itemId={:?}", item.id).to_string())?;
        let time = utc.with_timezone(&tz);
        let date = format!("{}", time.format("%Y%m%d"));
        match map.get_mut(&date) {
            Some(list) => {
                list.push(item.clone());
            }
            None => {
                map.insert(date, vec![item.clone()]);
            }
        };
    }
    map.sort_keys();
    Ok(map)
}

// Group items by dates according to filter param
pub fn group_by_metrics(
    metrics: &str,
    items: &Vec<Item>,
) -> Result<IndexMap<String, Vec<Item>>, String> {
    let mut map: IndexMap<String, Vec<Item>> = IndexMap::new();

    for item in items {
        let val_opt = &item.metrics.get(metrics);

        match val_opt.and_then(|v| v.as_str()) {
            Some(val) => {
                match map.get_mut(val) {
                    Some(list) => {
                        list.push(item.clone());
                    }
                    None => {
                        map.insert(String::from(val), vec![item.clone()]);
                    }
                };
            }
            None => {}
        };
    }
    Ok(map)
}
