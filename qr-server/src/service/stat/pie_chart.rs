use std::vec;

use indexmap::IndexMap;
use qr_model::Item;
use qr_repo::select_all_items;
use rocket::serde;
use rusqlite::Connection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    common::{group_by_dates, group_by_metrics, sum, KeyValueType, SeriesQuery},
    query::build_clauses_and_params,
    CommonFilter,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PieChartCateQuery {
    pub r#type: KeyValueType,
    // Required if type == Metrics
    pub metrics: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PieChartRequest {
    pub filter: CommonFilter,
    pub cate: PieChartCateQuery,
    pub series: Vec<SeriesQuery>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PieSeries {
    pub query: SeriesQuery,
    pub data: IndexMap<String, Decimal>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PieChartResult {
    pub series: Vec<PieSeries>,
}

// Query pie chart
pub fn query_pie_chart(conn: &Connection, req: &PieChartRequest) -> Result<PieChartResult, String> {
    let (clauses, params) = build_clauses_and_params(conn, &req.filter)?;
    let items = select_all_items(conn, clauses, params).map_err(|e| {
        log::error!("Failed to query items: {:?}", e);
        "Failed to query items".to_string()
    })?;
    log::debug!("{} items found for pie chart", items.len());
    let grouped_items = group_by(&req.cate, &req.filter, &items)?;
    log::debug!("{} grouped items of pie chart", grouped_items.len());
    let mut series: Vec<PieSeries> = vec![];
    for s in &req.series {
        let series_item = calc_series(s, &grouped_items)?;
        series.push(series_item);
    }
    Ok(PieChartResult { series })
}

fn group_by(
    category: &PieChartCateQuery,
    filter: &CommonFilter,
    items: &Vec<Item>,
) -> Result<IndexMap<String, Vec<Item>>, String> {
    match category.r#type {
        KeyValueType::Date => group_by_dates(filter, items),
        KeyValueType::Metrics => {
            let metrics = category
                .metrics
                .clone()
                .ok_or("Metrics key is required".to_string())?;
            group_by_metrics(&metrics, items)
        }
    }
}

fn calc_series(
    series: &SeriesQuery,
    grouped: &IndexMap<String, Vec<Item>>,
) -> Result<PieSeries, String> {
    let metrics = &series.metrics;
    let mut data: IndexMap<String, Decimal> = IndexMap::new();
    for (k, items) in grouped {
        let val = sum(&items, &metrics)?;
        data.insert(k.to_string(), val);
    }
    log::debug!("pie_series_sum: metrics={}, result={:?}", metrics, data);
    Ok(PieSeries {
        query: series.clone(),
        data,
    })
}
