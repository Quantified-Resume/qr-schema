use dateparser::DateTimeUtc;
use serde_json::{self, from_str, Map, Value};

pub fn date_from_sql(res: rusqlite::Result<String>) -> Option<i64> {
    let col_val = match res {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to get col from sql: {}", e);
            return None;
        }
    };
    match col_val.parse::<DateTimeUtc>() {
        Ok(val) => Some(val.0.timestamp_millis()),
        Err(e) => {
            log::error!("Time parse failure: {}", e);
            None
        }
    }
}

pub fn str_from_sql(res: rusqlite::Result<String>) -> Option<String> {
    match res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

pub fn json_map_from_sql(res: rusqlite::Result<String>) -> Option<Map<String, Value>> {
    let json_str = match res {
        Ok(v) => v,
        Err(_) => return None,
    };
    let json_res: Result<Map<String, serde_json::Value>, serde_json::Error> = from_str(&json_str);
    match json_res {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}
