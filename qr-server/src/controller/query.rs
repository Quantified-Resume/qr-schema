use super::common::HttpErrorJson;
use crate::{
    controller::common::RocketState,
    get_conn_lock,
    service::{
        query_group, query_indicator, query_line_chart, query_plain, CommonFilter, GroupRequest,
        GroupResult, IndicatorRequest, LineChartRequest, LineChartResult, PlainResult,
    },
};
use rocket::{post, serde::json::Json, State};
use rust_decimal::Decimal;

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

#[post("/indicator", data = "<body>", format = "application/json")]
pub fn get_indicator(
    body: Json<IndicatorRequest>,
    state: &State<RocketState>,
) -> Result<Json<Decimal>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    query_indicator(&conn, &body.0)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}

#[post("/line-chart", data = "<body>", format = "application/json")]
pub fn get_line_chart(
    body: Json<LineChartRequest>,
    state: &State<RocketState>,
) -> Result<Json<LineChartResult>, HttpErrorJson> {
    let conn = get_conn_lock!(state.conn);
    query_line_chart(&conn, &body.0)
        .map(Json)
        .map_err(HttpErrorJson::from_msg)
}
