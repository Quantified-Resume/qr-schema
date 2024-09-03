mod bucket;
mod common;

use std::sync::Mutex;

use common::RocketState;
use rocket::{routes, Build, Config, Rocket};
use rusqlite::Connection;

pub fn register_controllers(conn: Mutex<Connection>) -> Rocket<Build> {
    let ctx = RocketState::new(conn);
    let rocket = rocket::custom(Config::debug_default()).manage(ctx).mount(
        "/api/0/bucket",
        routes![
            bucket::create,
            bucket::list_all,
            bucket::get_detail,
            bucket::modify_detail,
        ],
    );
    rocket
}
