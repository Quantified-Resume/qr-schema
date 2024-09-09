pub mod controller;
pub mod service;

use std::sync::Mutex;

use crate::controller::register_controllers;
use rusqlite::Connection;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Init
    let conn_instance = Connection::open("test.db").expect("Failed to open database");
    qr_repo::init_tables(&conn_instance);
    // Register web api
    let conn = Mutex::new(conn_instance);
    let rocket = register_controllers(conn);
    // Launch
    rocket.launch().await?;
    Ok(())
}
