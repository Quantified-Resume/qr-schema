use super::common::HttpErrorJson;
use crate::{
    controller::common::RocketState,
    get_conn_lock,
    service::{create_item, BucketKey},
};
use qr_model::Item;
use rocket::{post, serde::json::Json, State};
use serde::Deserialize;
use serde_json::{Map, Value};

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
    let mut conn = get_conn_lock!(state.conn);
    let tx = match conn.transaction() {
        Err(e) => {
            log::error!("Failed to create transaction: {}", e);
            return Err(HttpErrorJson::sys_busy(e));
        }
        Ok(tx) => tx,
    };
    match create_item(&tx, &body.bucket, &body.to_item()) {
        Ok(id) => match tx.commit() {
            Ok(_) => Ok(Json(id)),
            Err(e) => Err(HttpErrorJson::sys_busy(e)),
        },
        Err(e) => match tx.rollback() {
            Ok(_) => Err(HttpErrorJson::from_err(&e, e.to_string())),
            Err(txe) => Err(HttpErrorJson::sys_busy(txe)),
        },
    }
}
