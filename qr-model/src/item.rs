use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::tag::Tag;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum ItemType {
    EVENT,
    RECORD,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct NamingEntity {
    id: Option<u64>,
    name: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum RecordMetricsType {
    TIME,
    COUNT,
    AMOUNT,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct RecordMetrics {
    #[serde(rename = "type")]
    type_: RecordMetricsType,
    // u64 for TIME and COUNT, decimal for Amount
    value: Value,
    // Only available for AMOUNT
    currency: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    id: Option<u64>,
    ref_key: String,
    #[serde(rename = "type")]
    type_: ItemType,
    timestamp: DateTime<Utc>,
    // Duration of item, required for EVENT
    duration: Option<u64>,
    name: Option<String>,
    action: String,
    tags: Option<Vec<Tag>>,
    place: Option<NamingEntity>,
    joiners: Option<Vec<NamingEntity>>,
    // Value list of item, required for RECORD
    metrics: Option<Vec<RecordMetrics>>,
    payload: Option<Map<String, Value>>,
}
