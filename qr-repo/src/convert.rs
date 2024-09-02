use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{Error, Result};
use serde_json::{self, from_str, Map, Value};

pub fn date_from_sql(res: Result<String, Error>) -> Option<DateTime<Utc>> {
    let col_val = res.unwrap();
    let res = NaiveDateTime::parse_from_str(&col_val, "%Y-%m-%d %H:%M:%S")
        .map(|time| DateTime::<Utc>::from_naive_utc_and_offset(time, Utc));
    match res {
        Ok(date) => Some(date.to_utc()),
        Err(e) => panic!("Time parse failure {}", e),
    }
}

pub fn str_from_sql(res: Result<String, Error>) -> Option<String> {
    match res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

pub fn json_map_from_sql(res: Result<String, Error>) -> Option<Map<String, Value>> {
    if res.is_err() {
        return None;
    }
    let json_str = res.unwrap();
    let json_res: Result<Map<String, serde_json::Value>, serde_json::Error> = from_str(&json_str);
    match json_res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}
