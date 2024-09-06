mod bucket;
mod common;
mod item;

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
            ],
        )
        .mount("/api/0/item", routes![item::create]);
    rocket
}
