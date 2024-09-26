use super::{
    common::{HttpErrorJson, RocketState},
    item,
};
use crate::{
    get_conn_lock,
    service::{self, check_bucket_exist, create_bucket, remove_bucket},
};
use itertools::Itertools;
use qr_model::{Bucket, BucketStatus, Builtin, Item};
use qr_repo::{select_all_buckets, update_bucket, BucketQuery};
use rocket::{delete, get, post, put, serde::json::Json, State};
use rusqlite::{Connection, TransactionBehavior};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::str::FromStr;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    name: String,
    desc: Option<String>,
    builtin: Option<Builtin>,
    builtin_ref_id: Option<String>,
    payload: Option<Map<String, Value>>,
}

impl CreateRequest {
    pub fn to_bucket(&self) -> Bucket {
        Bucket {
            id: None,
            no: None,
            name: self.name.clone(),
            builtin: self.builtin.clone(),
            builtin_ref_id: self.builtin_ref_id.clone(),
            status: BucketStatus::Enabled,
            desc: self.desc.clone(),
            url: None,
            tag: None,
            created: None,
            last_modified: None,
            payload: self.payload.clone(),
        }
    }
}

#[post("/", data = "<body>", format = "application/json")]
pub fn create(
    body: Json<CreateRequest>,
    state: &State<RocketState>,
) -> Result<Json<i64>, HttpErrorJson> {
    let request = body.into_inner();
    let mut bucket = request.to_bucket();
    let mut conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| HttpErrorJson::sys_busy(e))?;

    match create_bucket(&tx, &mut bucket) {
        Ok(id) => tx
            .commit()
            .map_err(|e| HttpErrorJson::sys_busy(e))
            .map(|_| Json(id)),
        Err(e) => tx
            .rollback()
            .map_err(|e| HttpErrorJson::sys_busy(e))
            .and_then(|_| Err(HttpErrorJson::from_msg(e))),
    }
}

#[get("/?<bt>&<bt_rid>")]
pub fn list_all(
    bt: Option<&str>,
    // RefId of builtin
    bt_rid: Option<&str>,
    state: &State<RocketState>,
) -> Result<Json<Vec<Bucket>>, HttpErrorJson> {
    let builtin = match bt {
        None => None,
        Some(s) => match Builtin::from_str(s) {
            Ok(v) => Some(v),
            Err(_) => {
                log::info!("Invalid builtin param: b={}", s);
                return Err(HttpErrorJson::invalid_param("Invalid param: b"));
            }
        },
    };

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    let query = BucketQuery {
        ref_id: bt_rid.filter(|v| !v.is_empty()).map(String::from),
        builtin,
    };
    select_all_buckets(&conn, query).map(Json).map_err(|e| {
        log::error!("Error to query all buckets: builtin={:?}", bt);
        HttpErrorJson::sys_busy(e)
    })
}

#[get("/<bucket_id>")]
pub fn get_detail(
    bucket_id: i64,
    state: &State<RocketState>,
) -> Result<Json<Bucket>, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    check_bucket_exist(&conn, bucket_id)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}

#[derive(Deserialize, Debug)]
pub struct ModifyRequest {
    pub name: String,
    pub desc: Option<String>,
    pub payload: Option<Map<String, Value>>,
}

#[put("/<bucket_id>", data = "<body>", format = "application/json")]
pub fn modify(
    bucket_id: i64,
    body: Json<ModifyRequest>,
    state: &State<RocketState>,
) -> Result<(), HttpErrorJson> {
    let request = body.into_inner();
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    let mut exist = check_bucket_exist(&conn, bucket_id).map_err(HttpErrorJson::from_msg)?;
    exist.name = request.name;
    exist.desc = request.desc;
    exist.payload = request.payload;
    update_bucket(&conn, &exist).map_err(|e| HttpErrorJson::from_err("Failed to save bucket", e))
}

#[delete("/<bucket_id>?<force>")]
pub fn remove(
    bucket_id: i64,
    force: Option<bool>,
    state: &State<RocketState>,
) -> Result<(), HttpErrorJson> {
    let mut conn = get_conn_lock!(state.conn);
    remove_bucket(&mut conn, bucket_id, force.unwrap_or(false)).map_err(HttpErrorJson::from_msg)
}

#[post("/<bid>/item", data = "<body>", format = "application/json")]
pub fn batch_create_items(
    bid: i64,
    body: Json<Vec<item::CreateRequest>>,
    state: &State<RocketState>,
) -> Result<(), HttpErrorJson> {
    let mut conn = get_conn_lock!(state.conn);
    let items = body.0.iter().map(|r| r.to_item()).collect_vec();
    service::batch_create_item(&mut conn, bid, items)
        .map(|_| ())
        .map_err(HttpErrorJson::from_msg)
}

#[get("/<bid>/item", format = "application/json")]
pub fn list_all_items(
    bid: i64,
    state: &State<RocketState>,
) -> Result<Json<Vec<Item>>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    service::list_item_by_bucket_id(&conn, bid)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}

#[get("/<bid>/chart", format = "application/json")]
pub fn list_supported_chart_series(
    bid: i64,
    state: &State<RocketState>,
) -> Result<Json<Vec<qr_model::ChartSeries>>, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, Connection> = get_conn_lock!(state.conn);
    let bucket = check_bucket_exist(&conn, bid).map_err(HttpErrorJson::from_msg)?;
    service::list_series_by_bucket(&conn, &bucket)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}
