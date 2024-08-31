use qr_model::Bucket;
use qr_repo::select_bucket_by_id;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};
use std::collections::HashMap;

use crate::get_conn_lock;

use super::common::{HttpErrorJson, RocketState};

#[get("/")]
pub fn query_bucket_page() -> Result<Json<HashMap<String, Bucket>>, HttpErrorJson> {
    Ok(Json(HashMap::new()))
}

#[get("/<bucket_id>")]
pub fn query_bucket_detail(
    bucket_id: &str,
    state: &State<RocketState>,
) -> Result<Bucket, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    let id = bucket_id.parse::<i64>().unwrap();

    match select_bucket_by_id(&conn, id) {
        Some(val) => Ok(val),
        None => Err(HttpErrorJson::new(Status::BadRequest, "Bucket not found".to_string())),
    }
}
