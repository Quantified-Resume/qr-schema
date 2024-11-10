use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Metrics;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MetaItem {
    value: serde_json::Value,
    label: String,
}

impl MetaItem {
    pub fn from_metrics(metrics: &Metrics) -> MetaItem {
        let value = serde_json::Value::from_str(&metrics.field).unwrap_or_else(|e| {
            log::warn!("Invalid field of metrics:{}", e);
            serde_json::Value::Null
        });
        MetaItem {
            value,
            label: metrics.label.to_string(),
        }
    }
}
