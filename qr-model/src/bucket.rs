use std::fmt::{Display, Error};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{tag, Builtin};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub id: Option<u64>,
    pub no: Option<i64>,
    pub name: String,
    pub builtin: Option<Builtin>,
    pub builtin_ref_id: Option<String>,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub tag: Option<Vec<tag::Tag>>,
    pub created: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
    pub payload: Option<Map<String, Value>>,
}

impl Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(&self) {
            Ok(json_str) => {
                write!(f, "{}", json_str).unwrap();
                Ok(())
            }
            Err(_e) => Err(Error::default()),
        }
    }
}

#[test]
fn test_bucket() {
    let bucket = Bucket {
        id: None,
        no: None,
        name: "BucketName".to_string(),
        builtin: Some(Builtin::BrowserTime),
        builtin_ref_id: None,
        desc: None,
        url: None,
        tag: None,
        created: Some(DateTime::default()),
        last_modified: Some(DateTime::default()),
        payload: None,
    };
    println!("Bucket={:?}", bucket)
}
