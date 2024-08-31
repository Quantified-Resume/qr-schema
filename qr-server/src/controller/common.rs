use std::sync::Mutex;

use rocket::http::Status;
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
