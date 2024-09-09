use super::common::HttpErrorJson;
use crate::{controller::common::RocketState, get_conn_lock, service::create_item};
use qr_model::{Bucket, BucketStatus, Builtin, Item};
use qr_repo::{select_bucket, select_bucket_by_builtin};
use rocket::{post, serde::json::Json, State};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{Map, Value};

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
    pub timestamp: i64,
    pub name: Option<String>,
    pub action: String,
    // Value list of item, required for RECORD
    pub metrics: Map<String, Value>,
    pub payload: Option<Map<String, Value>>,
}

impl CreateRequest {
    pub fn to_item(&self) -> Item {
        Item {
            id: None,
            ref_id: self.ref_id.clone(),
            timestamp: self.timestamp,
            name: self.name.clone(),
            action: self.action.clone(),
            metrics: self.metrics.clone(),
            payload: self.payload.clone(),
        }
    }
}

#[post("/", data = "<body>", format = "application/json")]
pub fn create(
    body: Json<CreateRequest>,
    state: &State<RocketState>,
) -> Result<Json<i64>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    let bucket = match check_bucket(&conn, &body.bucket) {
        Ok(bucket) => bucket,
        Err(e) => return Err(e),
    };
    match create_item(&conn, &bucket, &body.to_item()) {
        Ok(id) => Ok(Json(id)),
        Err(e) => Err(HttpErrorJson::from_err(&e, e.to_string())),
    }
}

pub fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Bucket, HttpErrorJson> {
    let bucket = match key.id {
        None => match &key.builtin {
            None => return Err(HttpErrorJson::bad_req("No bucket specified")),
            Some(val) => select_bucket_by_builtin(conn, &val, key.builtin_ref_id.as_deref()),
        },
        Some(id) => select_bucket(conn, id),
    };

    match bucket {
        None => Err(HttpErrorJson::bucket_not_found(key.id.unwrap_or(0))),
        Some(val) => match BucketStatus::Enabled == val.status {
            true => Ok(val),
            false => Err(HttpErrorJson::bucket_not_enabled(key.id.unwrap_or(0))),
        },
    }
}
