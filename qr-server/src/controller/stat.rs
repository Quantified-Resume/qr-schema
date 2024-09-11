use super::common::HttpErrorJson;
use crate::{controller::common::RocketState, service::BucketKey};
use rocket::{post, serde::json::Json, State};
use serde::Deserialize;

#[post("/profile", data = "<body>", format = "application/json")]
pub fn stat_profile(
    body: Json<String>,
    state: &State<RocketState>,
) -> Result<Json<i64>, HttpErrorJson> {
    Ok(Json(1))
}
