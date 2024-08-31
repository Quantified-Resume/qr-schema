pub mod controller;

use std::sync::Mutex;

use rusqlite::Connection;
use crate::controller::register_controllers;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let conn_instance = Connection::open("test.db").expect("Failed to open database");
    let conn = Mutex::new(conn_instance);
    let rocket = register_controllers(conn);

    rocket.launch().await?;
    Ok({})
}
