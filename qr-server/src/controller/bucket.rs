use qr_model::Bucket;
use qr_repo::select_bucket_by_id;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};

use crate::get_conn_lock;

use super::common::{HttpErrorJson, RocketState};

#[post("/", data = "<body>", format = "application/json")]
pub fn create(body: Json<Bucket>, state: &State<RocketState>) -> Result<i64, HttpErrorJson> {
    let conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);

    
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
        None => Err(HttpErrorJson::new(
            Status::BadRequest,
            "Bucket not found".to_string(),
        )),
    }
}
