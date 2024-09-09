use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(EnumString, AsRefStr, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum MetricsValueType {
    // With milliseconds
    Time,
    Count,
    Amount,
}

pub struct Metrics {
    /**
     * Name of field
     */
    pub field: String,
    /**
     * Value type
     */
    pub value_type: MetricsValueType,
    pub required: bool,
}

impl Metrics {
    pub fn new(field: &str, value_type: MetricsValueType, required: bool) -> Metrics {
        Metrics {
            field: String::from(field),
            value_type,
            required,
        }
    }
}
