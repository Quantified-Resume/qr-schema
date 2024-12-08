use super::common::HttpErrorJson;
use crate::{
    controller::common::RocketState,
    get_conn_lock,
    service::{create_item, BucketKey},
};
use qr_model::Item;
use qr_repo::select_item_by_ref_id;
use rocket::{get, post, serde::json::Json, State};
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    // Only required in single create
    pub bucket: Option<BucketKey>,
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
    let tx = conn.transaction().map_err(|e| HttpErrorJson::sys_busy(e))?;
    let bucket_key = body.bucket.clone().ok_or(HttpErrorJson::from_msg(
        "Bucket key is required".to_string(),
    ))?;
    match create_item(&tx, &bucket_key, &body.to_item()) {
        Ok(id) => tx
            .commit()
            .map_err(|e| HttpErrorJson::sys_busy(e))
            .map(|_| Json(id)),
        Err(e) => tx
            .rollback()
            .map_err(|txe| HttpErrorJson::sys_busy(txe))
            .and_then(|_| Err(HttpErrorJson::from_err(&e, e.to_string()))),
    }
}

#[get("/?<bid>&<rid>")]
pub fn get_detail_by_ref_id(
    bid: i64,
    rid: &str,
    state: &State<RocketState>,
) -> Result<Json<Item>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    let item = select_item_by_ref_id(&conn, bid, rid)
        .map_err(|e| {
            log::error!(
                "Failed to query item by refId: bucket_id={}, refId={}, err={}",
                bid,
                rid,
                e
            );
            HttpErrorJson::from_err("Failed", e)
        })?
        .ok_or_else(|| {
            log::error!("Item not found: bucket_id={}, ref_id={}", bid, rid);
            HttpErrorJson::not_found()
        })?;
    Ok(Json(item))
}

#[test]
pub fn test_create_request() {
    use serde_json::from_str;

    let json = r#"
        {"refId":"20240307crowdin.com","timestamp":1709740800000,"metrics":{"visit":4,"focus":227119,"host":"crowdin.com"},"action":"web_time","name":"crowdin.com","payload":{"date":"20240307","host":"crowdin.com","cid":"linux-google-chrome-1733631558570"}}
   "#;
    let param: CreateRequest = from_str(json).unwrap();

    assert!(param.metrics.len() == 3);
    assert_eq!("crowdin.com", param.metrics.get("host").unwrap());

    let item = param.to_item();
    assert_eq!("crowdin.com", item.metrics.get("host").unwrap());
}
