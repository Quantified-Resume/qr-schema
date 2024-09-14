pub mod config;
pub mod controller;
pub mod err;
pub mod logging;
pub mod service;

use crate::controller::register_controllers;
use crate::logging::init_logger;
use config::Config;
use rusqlite::Connection;
use std::sync::Mutex;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // 1. Parse config
    let conf = Config::parse();
    // 2. init logger
    init_logger(&conf);
    // 2. Init connection
    let conn_instance = Connection::open(conf.db_path).expect("Failed to open database");
    qr_repo::init_tables(&conn_instance);
    // 3. Register web api
    let conn = Mutex::new(conn_instance);
    let rocket_conf = rocket::Config::figment().merge(("port", conf.port));
    let rocket = register_controllers(conn).configure(rocket_conf);
    // Launch
    rocket.launch().await?;
    Ok(())
}
