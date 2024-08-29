use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum TagUsage {
    BUCKET,
    ITEM,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    id: Option<i64>,
    label: String,
    value: Option<String>,
    created: DateTime<Utc>,
    last_modified: DateTime<Utc>,
    usage: TagUsage,
}
