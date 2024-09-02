use qr_model::Bucket;
use qr_repo::insert_bucket;
use qr_repo::next_seq;
use qr_repo::select_bucket_by_id;
use qr_repo::Sequence;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use rusqlite::TransactionBehavior;

use super::common::{HttpErrorJson, RocketState};
use crate::get_conn_lock;

#[post("/", data = "<body>", format = "application/json")]
pub fn create(body: Json<Bucket>, state: &State<RocketState>) -> Result<Json<i64>, HttpErrorJson> {
    let mut bucket = body.into_inner();
    let mut conn: std::sync::MutexGuard<'_, rusqlite::Connection> = get_conn_lock!(state.conn);
    let tx_res = conn.transaction_with_behavior(TransactionBehavior::Immediate);
    if tx_res.is_err() {
        return Err(HttpErrorJson::internal_error("DB ERROR"));
    }
    let tx = tx_res.unwrap();
    let res: Result<i64, HttpErrorJson> = next_seq(&tx, Sequence::Bucket)
        .map_err(|_| HttpErrorJson::internal_error("Failed to gain transaction"))
        .and_then(|seq| {
            bucket.no = Some(seq);
            insert_bucket(&tx, &bucket)
                .map_err(|_| HttpErrorJson::new(Status::BadRequest, "Invalid param"))
        });
    tx.commit()
        .map_err(|_| HttpErrorJson::internal_error("Failed to commit"))
        .and(res)
        .and_then(|id| Ok(Json(id)))
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
