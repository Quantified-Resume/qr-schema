use std::fmt::{Display, Error};

use crate::{tag, Builtin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use strum::{AsRefStr, EnumString};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub id: Option<i64>,
    pub no: Option<i64>,
    pub name: String,
    pub builtin: Option<Builtin>,
    pub builtin_ref_id: Option<String>,
    pub status: BucketStatus,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub tag: Option<Vec<tag::Tag>>,
    pub created: Option<i64>,
    pub last_modified: Option<i64>,
    pub payload: Option<Map<String, Value>>,
}
impl Bucket {
    pub fn default_builtin(builtin: &Builtin, ref_id: Option<String>) -> Self {
        Bucket {
            id: None,
            no: None,
            name: builtin.get_default_name(),
            builtin: Some(builtin.clone()),
            builtin_ref_id: ref_id,
            status: BucketStatus::default(),
            desc: None,
            url: None,
            tag: None,
            created: None,
            last_modified: None,
            payload: None,
        }
    }
}

impl Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(&self) {
            Ok(json_str) => write!(f, "{}", json_str),
            Err(_e) => Err(Error::default()),
        }
    }
}

#[derive(
    EnumString, AsRefStr, Serialize, Deserialize, JsonSchema, Clone, Debug, Default, PartialEq, Eq,
)]
pub enum BucketStatus {
    #[default]
    Enabled,
    Disabled,
}
