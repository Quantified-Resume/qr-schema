use super::common::HttpErrorJson;
use crate::{
    controller::common::RocketState,
    get_conn_lock,
    service::{query_stat, QueryRequest},
};
use rocket::{post, serde::json::Json, State};

#[post("/profile", data = "<body>", format = "application/json")]
pub fn stat_profile(
    body: Json<QueryRequest>,
    state: &State<RocketState>,
) -> Result<Json<i64>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    query_stat(&conn, body.0)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}
