use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::Error;
use serde_json::{self, from_value, json, Map, Value};

pub fn date_from_sql(res: Result<String, Error>) -> Option<DateTime<Utc>> {
    let col_val = res.unwrap();
    let res = NaiveDateTime::parse_from_str(&col_val, "%Y-%m-%d %H:%M:%S")
        .map(|time| DateTime::<Utc>::from_naive_utc_and_offset(time, Utc));
    match res {
        Ok(date) => Some(date.to_utc()),
        Err(e) => panic!("Time parse failure {}", e),
    }
}

pub fn json_map_from_sql(res: Result<String, Error>) -> Option<Map<String, Value>> {
    let json_value = match res.map(|val: String| json!(val)) {
        Ok(res) => Some(res),
        Err(_) => None,
    };
    if json_value.is_none() {
        return None;
    }
    let val = json_value.unwrap();
    let from_res: Result<Map<String, Value>, serde_json::Error> = from_value(val);
    match from_res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}
