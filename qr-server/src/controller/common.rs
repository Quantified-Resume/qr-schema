use std::fmt::Debug;
use std::io::Cursor;
use std::sync::Mutex;

use rocket::http::ContentType;
use rocket::response::{self};
use rocket::{http::Status, request::Request, response::Responder, Response};
use rusqlite::Connection;
use serde::Serialize;

pub struct RocketState {
    pub conn: Mutex<Connection>,
}

impl RocketState {
    pub fn new(conn: Mutex<Connection>) -> Self {
        RocketState { conn }
    }
}

#[derive(Serialize, Debug)]
pub struct HttpErrorJson {
    #[serde(skip_serializing)]
    status: Status,
    message: String,
}

impl HttpErrorJson {
    pub fn new(status: Status, msg: &str) -> Self {
        HttpErrorJson {
            status,
            message: String::from(msg),
        }
    }

    pub fn from_err<E>(msg: &str, err: E) -> Self
    where
        E: Debug,
    {
        println!("Error occurred: {:?}", err);
        HttpErrorJson::new(Status::InternalServerError, msg)
    }
}

impl<'r> Responder<'r, 'static> for HttpErrorJson {
    fn respond_to(self, _: &Request) -> response::Result<'static> {
        // TODO: Fix unwrap
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .status(self.status)
            .sized_body(body.len(), Cursor::new(body))
            .header(ContentType::new("application", "json"))
            .ok()
    }
}

#[macro_export]
macro_rules! get_conn_lock {
    ($lock:expr) => {
        match $lock.lock() {
            Ok(r) => r,
            Err(e) => {
                use rocket::http::Status;
                println!("Server is busy, error={}", e);
                let err = HttpErrorJson::new(Status::ServiceUnavailable, "Service available");
                return Err(err);
            }
        }
    };
}
