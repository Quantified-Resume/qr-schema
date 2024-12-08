use std::str::FromStr;

use dateparser::DateTimeUtc;
use qr_model::{Metrics, MetricsValueType};
use rust_decimal::Decimal;
use rusty_money::iso::{self, Currency};
use serde_json::{Map, Value};

pub fn check_metrics(
    def: Vec<Metrics>,
    metrics_map: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    let mut valid_value = Map::new();
    for metrics in def {
        let val = metrics_map.get(&metrics.field);
        check_each(&metrics, val)?.and_then(|v| valid_value.insert(metrics.field, v.clone()));
    }
    Ok(valid_value)
}

fn check_each(metrics: &Metrics, mut val_opt: Option<&Value>) -> Result<Option<Value>, String> {
    let val = match val_opt.take_if(|v| !v.is_null()) {
        None => match metrics.required {
            true => return Err(format!("Metrics field [{}] is required", metrics.field)),
            false => return Ok(None),
        },
        Some(v) => v,
    };

    let type_check_res = match metrics.value_type {
        MetricsValueType::Time => check_time(val),
        MetricsValueType::Count => check_count(val),
        MetricsValueType::Amount => check_amount(val),
        MetricsValueType::Enum => check_enum(val),
    };
    type_check_res.map(|val| Some(val)).map_err(String::from)
}

fn check_time(val: &Value) -> Result<Value, String> {
    if val.is_string() {
        let val_str = val.to_string();
        val_str
            .parse::<DateTimeUtc>()
            .map_err(|e| {
                log::error!(
                    "Failed to parse date time from string for metrics: str={}, err={}",
                    val_str,
                    e
                );
                "Failed to parse time".to_string()
            })
            .map(|utc| Value::from(utc.0.timestamp_millis()))
    } else if val.is_number() {
        Ok(val.clone())
    } else {
        Err("Invalid value type of time".to_string())
    }
}

fn check_count(val: &Value) -> Result<Value, String> {
    if val.is_number() {
        Ok(val.clone())
    } else {
        Err("Invalid value type of count".to_string())
    }
}

fn check_amount(val: &Value) -> Result<Value, String> {
    if val.is_string() {
        let val_str = val.to_string();
        let currency = iso::find(&val_str[0..3]).ok_or("Invalid currency".to_string())?;
        let value = Decimal::from_str_radix(&val_str[3..], 10)
            .map_err(|_| format!("Failed to parse value: {}", val_str))?;
        return Ok(amount_of(currency, value));
    } else if val.is_object() {
        let map = val.as_object().ok_or("Invalid object".to_string())?;
        let currency = map
            .get("currency")
            .take_if(|v| v.is_string())
            .and_then(|c| iso::find(&c.to_string()))
            .ok_or("Failed to find currency".to_string())?;
        let value = map
            .get("value")
            .ok_or("Failed to find currency".to_string())?;
        if value.is_string() || value.is_number() {
            let decimal = Decimal::from_str_radix(&value.to_string(), 10)
                .map_err(|_| format!("Failed to parse value: {}", 1))?;
            return Ok(amount_of(currency, decimal));
        }
        return Err(format!("Invalid value: {}", value));
    }
    Err("Not supported amount object".to_string())
}

fn amount_of(currency: &Currency, value: Decimal) -> Value {
    let mut map = Map::new();
    map.insert(
        "currency".to_string(),
        Value::String(currency.iso_alpha_code.to_string()),
    );
    map.insert("value".to_string(), Value::String(value.to_string()));
    Value::Object(map)
}

fn check_enum(val: &Value) -> Result<Value, String> {
    let enum_str = val.to_string();
    Value::from_str(&enum_str).map_err(|e| {
        log::error!("Unexpected error: {}", e);
        "Unexpected error: check_enum".to_string()
    })
}
