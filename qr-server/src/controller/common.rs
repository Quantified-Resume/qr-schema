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
    pub fn new(status: Status, msg: String) -> Self {
        HttpErrorJson {
            status,
            message: msg,
        }
    }

    pub fn system_error(status: Status) -> Self {
        HttpErrorJson::new(status, "System error".to_string())
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
                let err = HttpErrorJson::system_error(Status::ServiceUnavailable);
                return Err(err);
            }
        }
    };
}
