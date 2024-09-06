use qr_model::{Bucket, BucketStatus, Builtin, Item, ItemType, NamingEntity, RecordMetrics};
use qr_repo::{insert_item, select_bucket, select_bucket_by_builtin};
use rocket::{post, serde::json::Json, State};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{controller::common::RocketState, get_conn_lock};

use super::{bucket, common::HttpErrorJson};

#[derive(Deserialize, Debug)]
pub struct BucketKey {
    id: Option<i64>,
    builtin: Option<Builtin>,
    builtin_ref_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateRequest {
    pub bucket: BucketKey,

    pub ref_id: String,
    #[serde(rename = "type")]
    pub type_: ItemType,
    pub timestamp: i64,
    // Duration of item, required for EVENT
    pub duration: Option<i64>,
    pub name: Option<String>,
    pub action: String,
    pub place: Option<NamingEntity>,
    pub joiners: Option<Vec<NamingEntity>>,
    // Value list of item, required for RECORD
    pub metrics: Option<Vec<RecordMetrics>>,
    pub payload: Option<Map<String, Value>>,
}

impl CreateRequest {
    pub fn to_item(&self) -> Result<Item, HttpErrorJson> {
        let item = Item {
            id: None,
            ref_id: self.ref_id.clone(),
            type_: self.type_.clone(),
            timestamp: self.timestamp,
            duration: self.duration,
            name: self.name.clone(),
            action: self.action.clone(),
            metrics: self.metrics.clone(),
            payload: self.payload.clone(),
        };
        match self.type_ {
            ItemType::Event => self
                .duration
                .ok_or(HttpErrorJson::bad_req("Duration must not be empty"))
                .map(|_| item),
            ItemType::Record => self
                .metrics
                .clone()
                .and_then(|v| match v.len() > 0 {
                    true => Some(v),
                    false => None,
                })
                .ok_or(HttpErrorJson::bad_req("Metrics must not be empty"))
                .map(|_| item),
        }
    }
}

#[post("/", data = "<body>", format = "application/json")]
pub fn create(
    body: Json<CreateRequest>,
    state: &State<RocketState>,
) -> Result<Json<i64>, HttpErrorJson> {
    let item_res = body.to_item();
    if item_res.is_err() {
        return Err(item_res.unwrap_err());
    }
    let item = item_res.unwrap();

    let conn = get_conn_lock!(state.conn);
    let bucket_res = check_bucket(&conn, &body.bucket);
    bucket_res.and_then(|bucket| {
        insert_item(&conn, bucket.id.unwrap(), &item)
            .map_err(|e| HttpErrorJson::from_err("Failed to create item", e))
            .map(|item_id| Json(item_id))
    })
}

pub fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Bucket, HttpErrorJson> {
    let bucket = match key.id {
        None => match &key.builtin {
            None => None,
            Some(val) => select_bucket_by_builtin(conn, &val, key.builtin_ref_id.as_deref()),
        },
        Some(id) => select_bucket(conn, id),
    };

    match bucket {
        None => Err(HttpErrorJson::bucket_not_found(key.id.unwrap_or(0))),
        Some(val) => match BucketStatus::Enabled == val.status {
            true => Ok(val),
            false => Err(HttpErrorJson::bucket_not_found(key.id.unwrap_or(0))),
        },
    }
}
