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
        match check_each(&metrics, val) {
            Ok(opt) => opt.and_then(|v| valid_value.insert(metrics.field, v.clone())),
            Err(e) => return Err(e),
        };
    }
    Ok(valid_value)
}

fn check_each(metrics: &Metrics, mut val_opt: Option<&Value>) -> Result<Option<Value>, String> {
    let val = match val_opt.take_if(|v| !v.is_null()) {
        None => match metrics.required {
            true => return Err(format! {"Metrics field [{}] is required", metrics.field}),
            false => return Ok(None),
        },
        Some(v) => v,
    };

    let type_check_res = match metrics.value_type {
        MetricsValueType::Time => check_time(val),
        MetricsValueType::Count => check_count(val),
        MetricsValueType::Amount => check_amount(val).map_err(|_| "Invalid amount".to_string()),
    };
    type_check_res.map(|val| Some(val))
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
    match val.is_number() {
        true => Ok(val.clone()),
        false => Err("Invalid value type of count".to_string()),
    }
}

fn check_amount(val: &Value) -> Result<Value, ()> {
    if val.is_string() {
        let val_str = val.to_string();
        return match iso::find(&val_str[0..3]) {
            Some(currency) => match Decimal::from_str_radix(&val_str[3..], 10) {
                Ok(value) => Ok(amount_of(currency, value)),
                Err(e) => {
                    log::error!("Failed to parse amount: {}, err={}", val_str, e);
                    Err(())
                }
            },
            None => Err(()),
        };
    } else if val.is_object() {
        let map = match val.as_object() {
            Some(m) => m,
            None => return Err(()),
        };
        let currency_opt = map
            .get("currency")
            .take_if(|v| v.is_string())
            .and_then(|c| iso::find(&c.to_string()));
        let currency = match currency_opt {
            Some(v) => v,
            None => return Err(()),
        };
        let value = match map.get("value") {
            Some(v) => v,
            None => return Err(()),
        };
        if value.is_string() || value.is_number() {
            return match Decimal::from_str_radix(&value.to_string(), 10) {
                Ok(decimal) => Ok(amount_of(currency, decimal)),
                Err(_) => Err(()),
            };
        }
        return Err(());
    }
    Err(())
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
