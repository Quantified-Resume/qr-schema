use super::common::HttpErrorJson;
use crate::{
    controller::common::RocketState,
    get_conn_lock,
    service::{query_chart, ChartQueryRequest, ChartResult},
};
use rocket::{post, serde::json::Json, State};

#[post("/chart", data = "<body>", format = "application/json")]
pub fn stat_profile(
    body: Json<ChartQueryRequest>,
    state: &State<RocketState>,
) -> Result<Json<ChartResult>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    query_chart(&conn, body.0)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}
