use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::tag;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub id: Option<u64>,
    pub no: Option<String>,
    pub name: String,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub tag: Option<Vec<tag::Tag>>,
    pub created: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub payload: Option<Map<String, Value>>,
}

#[test]
fn test_bucket() {
    let bucket = Bucket {
        id: None,
        no: None,
        name: "BucketName".to_string(),
        desc: None,
        url: None,
        tag: None,
        created: DateTime::default(),
        last_modified: DateTime::default(),
        payload: None,
    };
    println!("Bucket={:?}", bucket)
}
