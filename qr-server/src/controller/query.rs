use super::common::HttpErrorJson;
use crate::{
    controller::common::RocketState,
    get_conn_lock,
    service::{query_group, query_plain, CommonFilter, GroupRequest, GroupResult, PlainResult},
};
use rocket::{post, serde::json::Json, State};

#[post("/plain", data = "<body>", format = "application/json")]
pub fn list_items(
    body: Json<CommonFilter>,
    state: &State<RocketState>,
) -> Result<Json<PlainResult>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    query_plain(&conn, &body.0)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}

#[post("/group", data = "<body>", format = "application/json")]
pub fn list_group(
    body: Json<GroupRequest>,
    state: &State<RocketState>,
) -> Result<Json<GroupResult>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    query_group(&conn, &body.0)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}
