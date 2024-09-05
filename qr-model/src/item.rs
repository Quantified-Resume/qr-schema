use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use strum::{AsRefStr, EnumString};

use crate::tag::Tag;

#[derive(EnumString, AsRefStr, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum ItemType {
    Event,
    Record,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct NamingEntity {
    id: Option<i64>,
    name: String,
}

#[derive(EnumString, AsRefStr, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum RecordMetricsType {
    Time,
    Count,
    Amount,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct RecordMetrics {
    #[serde(rename = "type")]
    type_: RecordMetricsType,
    // i64 for TIME and COUNT, decimal for Amount
    value: Value,
    // Only available for AMOUNT
    currency: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Option<i64>,
    pub ref_id: String,
    #[serde(rename = "type")]
    pub type_: ItemType,
    pub timestamp: DateTime<Utc>,
    // Duration of item, required for EVENT
    pub duration: Option<i64>,
    pub name: Option<String>,
    pub action: String,
    pub tags: Option<Vec<Tag>>,
    pub place: Option<NamingEntity>,
    pub joiners: Option<Vec<NamingEntity>>,
    // Value list of item, required for RECORD
    pub metrics: Option<Vec<RecordMetrics>>,
    pub payload: Option<Map<String, Value>>,
}
