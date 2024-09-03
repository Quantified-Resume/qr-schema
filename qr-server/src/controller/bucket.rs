use qr_model::Bucket;
use qr_model::Builtin;
use qr_repo::insert_bucket;
use qr_repo::next_seq;
use qr_repo::select_all_buckets;
use qr_repo::select_bucket_by_builtin;
use qr_repo::select_bucket_by_id;
use qr_repo::update_bucket_by_id;
use qr_repo::Sequence;
use rocket::http::Status;
use rocket::put;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rusqlite::Connection;
use rusqlite::TransactionBehavior;
use serde::Deserialize;
use serde_json::Map;
use serde_json::Value;

use super::common::{HttpErrorJson, RocketState};
use crate::get_conn_lock;

#[derive(Deserialize, Debug)]
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
    // 1. check param
    let builtin_check =
        check_builtin(&conn, &bucket).map_err(|msg| HttpErrorJson::new(Status::BadRequest, &msg));
    if builtin_check.is_err() {
        return Err(builtin_check.unwrap_err());
    }
    let tx_res = conn.transaction_with_behavior(TransactionBehavior::Immediate);
    if tx_res.is_err() {
        return Err(HttpErrorJson::from_err("DB ERROR", tx_res.unwrap_err()));
    }
    let tx = tx_res.unwrap();
    let res = next_seq(&tx, Sequence::Bucket)
        .map_err(|e| HttpErrorJson::from_err("Failed to gain transaction", e))
        .and_then(|seq| {
            bucket.no = Some(seq);
            insert_bucket(&tx, &bucket).map_err(|e| HttpErrorJson::from_err("Failed to save", e))
        });
    match res {
        Ok(id) => tx
            .commit()
            .map_err(|e| HttpErrorJson::from_err("Failed to commit", e))
            .map(|_| Json(id)),
        Err(e) => tx
            .rollback()
            .map_err(|e| HttpErrorJson::from_err("Failed to rollback", e))
            .and_then(|_| Err(e)),
    }
}

fn check_builtin(conn: &Connection, bucket: &Bucket) -> Result<(), String> {
    let builtin = &bucket.builtin;
    if builtin.is_none() {
        // Customized bucket, not check
        return Ok(());
    }
    let b = builtin.clone().unwrap();
    let ref_id = bucket.builtin_ref_id.clone();
    let exist_one = select_bucket_by_builtin(conn, &b, ref_id.as_deref());
    match b.is_multiple() {
        true => match ref_id {
            None => Err("Ref ID is required for this builtin".to_string()),
            Some(_) => match exist_one {
                Some(v) => Err(format!("This bucket already exists: no={}", v.no.unwrap())),
                None => Ok(()),
            },
        },
        false => match exist_one {
            Some(v) => Err(format!("This bucket already exists: no={}", v.no.unwrap())),
            None => Ok(()),
        },
    }
}

#[get("/")]
pub fn list_all(state: &State<RocketState>) -> Result<Json<Vec<Bucket>>, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    Ok(Json(select_all_buckets(&conn)))
}

#[get("/<bucket_id>")]
pub fn get_detail(
    bucket_id: i64,
    state: &State<RocketState>,
) -> Result<Json<Bucket>, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);

    match select_bucket_by_id(&conn, bucket_id) {
        Some(val) => Ok(Json(val)),
        None => Err(HttpErrorJson::bucket_not_found(bucket_id)),
    }
}

#[derive(Deserialize, Debug)]
pub struct ModifyRequest {
    pub name: String,
    pub desc: Option<String>,
}

#[put("/<bucket_id>", data = "<body>", format = "application/json")]
pub fn modify_detail(
    bucket_id: i64,
    body: Json<ModifyRequest>,
    state: &State<RocketState>,
) -> Result<(), HttpErrorJson> {
    let request = body.into_inner();
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    select_bucket_by_id(&conn, bucket_id)
        .ok_or(HttpErrorJson::bucket_not_found(bucket_id))
        .and_then(|exist| {
            let mut clone = exist.clone();
            clone.name = request.name;
            clone.desc = request.desc;
            update_bucket_by_id(&conn, &clone)
                .map_err(|e| HttpErrorJson::from_err("Failed to save bucket", e))
        })
        .map(|_| ())
}
