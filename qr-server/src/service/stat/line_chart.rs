use std::{collections::HashMap, vec};

use chrono::{DateTime, Datelike, Days, FixedOffset};
use indexmap::IndexMap;
use qr_model::Item;
use qr_repo::select_all_items;
use rocket::serde;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{common::sum, query::build_clauses_and_params, CommonFilter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum XValueType {
    Date,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineChartQueryX {
    pub r#type: XValueType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineChartQueryY {
    pub metrics: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LineChartRequest {
    pub filter: CommonFilter,
    pub x: LineChartQueryX,
    pub y: Vec<LineChartQueryY>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineChartResultX {
    pub value: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineChartItemY {
    pub x: String,
    pub y: Decimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LineChartResult {
    pub x: LineChartResultX,
    pub y: HashMap<String, Vec<LineChartItemY>>,
}

pub fn query_line_chart(
    conn: &Connection,
    req: &LineChartRequest,
) -> Result<LineChartResult, String> {
    let (clauses, params) = build_clauses_and_params(conn, &req.filter)?;
    let items = select_all_items(conn, clauses, params).map_err(|e| {
        log::error!("Failed to query items: {:?}", e);
        "Failed to query items".to_string()
    })?;

    let grouped_items = group_by(&req.x, &req.filter, &items)?;
    let x_values = calc_x_values(&req, &items)?;
    let y_values = calc_y_values(&req.y, &grouped_items)?;

    let res = LineChartResult {
        x: LineChartResultX { value: x_values },
        y: y_values,
    };
    Ok(res)
}

/// Group by x axis
fn group_by(
    x: &LineChartQueryX,
    filter: &CommonFilter,
    items: &Vec<Item>,
) -> Result<IndexMap<String, Vec<Item>>, String> {
    match x.r#type {
        XValueType::Date => group_by_dates(filter, items),
    }
}

fn group_by_dates(
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

fn calc_x_values(req: &LineChartRequest, items: &Vec<Item>) -> Result<Vec<String>, String> {
    if items.is_empty() {
        return Ok(vec![]);
    };
    let offset_secs = (req.filter.utc_offset * 60) as i32;
    let start_ts = req
        .filter
        .ts_start
        .or_else(|| items.iter().map(|i| i.timestamp).min())
        .ok_or("Failed to calc start date".to_string())?;
    let end_ts = req
        .filter
        .ts_end
        .or_else(|| items.iter().map(|i| i.timestamp).max())
        .ok_or("Failed to calc end date".to_string())?;
    let tz = FixedOffset::west_opt(offset_secs).ok_or("Invalid offset value".to_string())?;
    let mut start = DateTime::from_timestamp_millis(start_ts)
        .ok_or("Unexpected error")?
        .with_timezone(&tz);

    let end = DateTime::from_timestamp_millis(end_ts)
        .ok_or("Unexpected error")?
        .with_timezone(&tz);

    let day_1 = Days::new(1);
    let mut dates = vec![];
    while start.num_days_from_ce() <= end.num_days_from_ce() {
        let date = format!("{}", start.format("%Y%m%d"));
        dates.push(date);
        start = start
            .checked_add_days(day_1)
            .ok_or("Failed to iterate start date")?;
    }
    Ok(dates)
}

fn calc_y_values(
    y_queries: &Vec<LineChartQueryY>,
    grouped: &IndexMap<String, Vec<Item>>,
) -> Result<HashMap<String, Vec<LineChartItemY>>, String> {
    let mut map = HashMap::new();
    for y_query in y_queries {
        let metrics = &y_query.metrics;
        let mut values = vec![];
        for (x, items) in grouped {
            let y = sum(items, metrics)?;
            let value = LineChartItemY {
                x: x.to_string(),
                y,
            };
            values.push(value);
        }
        map.insert(metrics.to_string(), values);
    }
    Ok(map)
}
