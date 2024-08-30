pub mod controller;

use std::sync::Mutex;

use rusqlite::Connection;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let conn_instance = Connection::open("test.db").expect("Failed to open database");
    let conn = Mutex::new(conn_instance);
    let rocket = rocket::build().mount("/", routes![index]).ignite().await?;

    rocket.launch().await?;
    Ok({})
}
