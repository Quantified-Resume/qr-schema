mod bucket;
mod common;

use std::sync::Mutex;

use common::RocketState;
use rocket::{Build, Config, Rocket};
use rusqlite::Connection;

pub fn register_controllers(conn: Mutex<Connection>) -> Rocket<Build> {
    let ctx = RocketState::new(conn);
    let mut rocket = rocket::custom(Config::debug_default()).manage(ctx).mount(
        "/api/0/bucket",
        [bucket::query_bucket_page, bucket::query_bucket_detail],
    );
    rocket
}
