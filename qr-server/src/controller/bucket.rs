use std::str::FromStr;

use itertools::Itertools;
use qr_model::{Bucket, BucketStatus, Builtin};
use qr_repo::{
    delete_bucket, delete_item_by_bucket_id, exist_item_by_bucket_id, select_all_buckets,
    select_bucket, update_bucket, BucketQuery,
};
use rocket::{delete, get, http::Status, post, put, serde::json::Json, State};
use rusqlite::{Connection, TransactionBehavior};
use serde::Deserialize;
use serde_json::{Map, Value};

use super::{
    common::{HttpErrorJson, RocketState},
    item,
};
use crate::{
    get_conn_lock,
    service::{self, create_bucket},
};

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
    let tx = match conn.transaction_with_behavior(TransactionBehavior::Immediate) {
        Ok(v) => v,
        Err(e) => return Err(HttpErrorJson::sys_busy(e)),
    };
    match create_bucket(&tx, &mut bucket) {
        Ok(id) => tx
            .commit()
            .map_err(|e| HttpErrorJson::sys_busy(e))
            .map(|_| Json(id)),
        Err(e) => tx
            .rollback()
            .map_err(|e| HttpErrorJson::sys_busy(e))
            .and_then(|_| Err(HttpErrorJson::from_msg(&e))),
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
                log::warn!("Invalid builtin param: b={}", s);
                return Err(HttpErrorJson::invalid_param("Invalid param: b"));
            }
        },
    };

    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    let query = BucketQuery {
        ref_id: bt_rid.filter(|v| !v.is_empty()).map(String::from),
        builtin,
    };
    match select_all_buckets(&conn, query) {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            log::error!("Error to query all buckets: builtin={:?}", bt);
            Err(HttpErrorJson::sys_busy(e))
        }
    }
}

#[get("/<bucket_id>")]
pub fn get_detail(
    bucket_id: i64,
    state: &State<RocketState>,
) -> Result<Json<Bucket>, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    check_bucket_exist(&conn, bucket_id).map(|val| Json(val))
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
    check_bucket_exist(&conn, bucket_id)
        .and_then(|exist| {
            let mut clone = exist.clone();
            clone.name = request.name;
            clone.desc = request.desc;
            clone.payload = request.payload;
            update_bucket(&conn, &clone)
                .map_err(|e| HttpErrorJson::from_err("Failed to save bucket", e))
        })
        .map(|_| ())
}

#[delete("/<bucket_id>?<force>")]
pub fn remove(
    bucket_id: i64,
    force: Option<bool>,
    state: &State<RocketState>,
) -> Result<(), HttpErrorJson> {
    let mut conn = get_conn_lock!(state.conn);
    let check_res = check_bucket_exist(&conn, bucket_id);
    if check_res.is_err() {
        return check_res.map(|_| ());
    }
    let exist_items = exist_item_by_bucket_id(&conn, bucket_id);
    if exist_items && !force.unwrap_or(false) {
        return Err(HttpErrorJson::new(
            Status::BadRequest,
            "Can't delete bucket with items",
        ));
    }
    let tx_res = conn.transaction_with_behavior(TransactionBehavior::Immediate);
    if tx_res.is_err() {
        return Err(HttpErrorJson::sys_busy(tx_res.unwrap_err()));
    }
    let tx = tx_res.unwrap();
    let res = delete_item_by_bucket_id(&tx, bucket_id).and_then(|cnt| {
        log::warn!("Deleted {} items of bucket[id={}]", cnt, bucket_id);
        delete_bucket(&tx, bucket_id).map(|_| ())
    });
    match res {
        Ok(_) => tx.commit().map_err(|e| HttpErrorJson::sys_busy(e)),
        Err(e) => match tx.rollback() {
            Ok(_) => Err(HttpErrorJson::sys_busy(e)),
            Err(txe) => Err(HttpErrorJson::sys_busy(txe)),
        },
    }
}

fn check_bucket_exist(conn: &Connection, bucket_id: i64) -> Result<Bucket, HttpErrorJson> {
    select_bucket(&conn, bucket_id).ok_or(HttpErrorJson::bucket_not_found(bucket_id))
}

#[post("/<bid>/item", data = "<body>", format = "application/json")]
pub fn batch_create_items(
    bid: i64,
    body: Json<Vec<item::CreateRequest>>,
    state: &State<RocketState>,
) -> Result<(), HttpErrorJson> {
    let mut conn = get_conn_lock!(state.conn);
    let items = body.0.iter().map(|r| r.to_item()).collect_vec();
    match service::batch_create_item(&mut conn, bid, items) {
        Ok(_) => Ok(()),
        Err(e) => Err(HttpErrorJson::from_msg(&e)),
    }
}
