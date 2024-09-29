mod bucket;
mod common;
mod item;
mod query;

use std::sync::Mutex;

use common::RocketState;
use rocket::{routes, Build, Config, Rocket};
use rusqlite::Connection;

pub fn register_controllers(conn: Mutex<Connection>) -> Rocket<Build> {
    let ctx = RocketState::new(conn);
    let rocket = rocket::custom(Config::debug_default())
        .manage(ctx)
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
                bucket::list_supported_chart_series,
            ],
        )
        .mount(
            "/api/0/item",
            routes![item::create, item::get_detail_by_ref_id],
        )
        .mount(
            "/api/0/query",
            routes![query::list_items, query::list_group],
        );
    rocket
}
