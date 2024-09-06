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
    created: Option<i64>,
    last_modified: Option<i64>,
    usage: TagUsage,
}
