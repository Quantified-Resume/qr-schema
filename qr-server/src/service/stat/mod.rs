mod common;
mod convert;
mod group;
mod indicator;
mod line_chart;
mod plain;
mod query;

pub use group::*;
pub use indicator::*;
pub use line_chart::*;
pub use plain::*;

use super::BucketKey;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Clone, Serialize, Deserialize, Debug)]
pub enum AggregationMethod {
    Sum,
    Average,
    Count,
}

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
pub struct CommonFilter {
    pub bucket: BucketKey,
    // Utc Offset with minutes
    pub utc_offset: i64,
    pub ts_start: Option<i64>,
    pub ts_end: Option<i64>,
    pub payload: Option<Vec<PayloadFilter>>,
    pub metrics: Option<Vec<String>>,
}
