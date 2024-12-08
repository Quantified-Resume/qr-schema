use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum MetricsValueType {
    // With milliseconds
    Time,
    Count,
    Amount,
    Enum,
}

pub struct Metrics {
    /**
     * Name of field
     */
    pub field: String,
    /**
     * Label of field
     */
    pub label: String,
    /**
     * Value type
     */
    pub value_type: MetricsValueType,
    pub required: bool,
}

impl Metrics {
    pub fn new(field: &str, label: &str, value_type: MetricsValueType, required: bool) -> Metrics {
        Metrics {
            field: String::from(field),
            label: String::from(label),
            value_type,
            required,
        }
    }
}
