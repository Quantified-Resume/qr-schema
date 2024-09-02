use qr_model::Bucket;
use qr_model::Builtin;
use qr_repo::insert_bucket;
use qr_repo::next_seq;
use qr_repo::select_bucket_by_id;
use qr_repo::Sequence;
use rocket::http::Status;
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
    let param_check = check_4_create(&conn, &bucket);
    if param_check.is_err() {
        return Err(HttpErrorJson::new(
            Status::BadRequest,
            &param_check.unwrap_err(),
        ));
    }
    let tx_res = conn.transaction_with_behavior(TransactionBehavior::Immediate);
    if tx_res.is_err() {
        return Err(HttpErrorJson::from_err("DB ERROR", tx_res.unwrap_err()));
    }
    let tx = tx_res.unwrap();
    let res: Result<i64, HttpErrorJson> = next_seq(&tx, Sequence::Bucket)
        .map_err(|e| HttpErrorJson::from_err("Failed to gain transaction", e))
        .and_then(|seq| {
            bucket.no = Some(seq);
            insert_bucket(&tx, &bucket).map_err(|e| HttpErrorJson::from_err("Invalid param", e))
        });
    tx.commit()
        .map_err(|e| HttpErrorJson::from_err("Failed to commit", e))
        .and(res)
        .and_then(|id| Ok(Json(id)))
}

fn check_4_create(conn: &Connection, bucket: &Bucket) -> Result<(), String> {
    match &bucket.builtin {
        None => Ok(()),
        Some(builtin) => {
            if !builtin.is_multiple() {
                return Ok(());
            }
            let builtin_ref_id = bucket.builtin_ref_id.clone();
            match builtin_ref_id {
                None => Err("Ref ID of this bucket must be empty"),
                Some => match select_bucket_by_id(conn, id) {
                    
                }
            }
            Ok(())
        }
    }
}

#[get("/")]
pub fn get_page() -> Result<Json<Vec<Bucket>>, HttpErrorJson> {
    Ok(Json(vec![]))
}

#[get("/<bucket_id>")]
pub fn get_detail(
    bucket_id: i64,
    state: &State<RocketState>,
) -> Result<Json<Bucket>, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);

    match select_bucket_by_id(&conn, bucket_id) {
        Some(val) => Ok(Json(val)),
        None => Err(HttpErrorJson::new(Status::BadRequest, "Bucket not found")),
    }
}
