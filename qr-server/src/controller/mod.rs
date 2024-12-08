mod bucket;
mod common;
mod item;
mod query;

use std::sync::Mutex;

use common::{HttpErrorJson, RocketState};
use rocket::{catch, catchers, http::Status, routes, Build, Config, Request, Rocket};
use rusqlite::Connection;

#[catch(500)]
fn handle_500(_status: Status, _req: &Request) -> HttpErrorJson {
    HttpErrorJson::from_msg("Unexpected error occurred".to_string())
}

pub fn register_controllers(conn: Mutex<Connection>) -> Rocket<Build> {
    let ctx = RocketState::new(conn);
    let rocket = rocket::custom(Config::debug_default())
        .manage(ctx)
        .register("/", catchers![handle_500])
        .mount(
            "/api/0/bucket",
            routes![
                bucket::create,
                bucket::modify,
                bucket::remove,
                bucket::get_detail,
                bucket::list_all,
                bucket::batch_create_items,
                bucket::list_all_items,
                bucket::list_all_metrics,
            ],
        )
        .mount(
            "/api/0/item",
            routes![item::create, item::get_detail_by_ref_id],
        )
        .mount(
            "/api/0/query",
            routes![
                query::list_items,
                query::list_group,
                query::get_indicator,
                query::get_line_chart,
                query::get_pie_chart,
            ],
        );
    rocket
}
