use std::sync::Mutex;

use rocket::{Build, Config, Rocket};
use rusqlite::Connection;

pub fn registerControllers(conn: Mutex<Connection>)->Rocket<Build>  {
    let mut rocket = rocket::custom(Config::debug_default())
        .manage(conn)
        .mount("/api/0/bucket", [
            bucket::query_bucket_page,
            bucket::query_bucket_detail,
        ]);
    rocket
}
