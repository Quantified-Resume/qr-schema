use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct NamingEntity {
    id: Option<i64>,
    name: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Option<i64>,
    pub ref_id: String,
    pub timestamp: i64,
    pub name: Option<String>,
    pub action: String,
    pub metrics: Map<String, Value>,
    pub payload: Option<Map<String, Value>>,
}
