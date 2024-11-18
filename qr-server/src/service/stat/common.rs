use std::{any::Any, str::FromStr};

use qr_model::Item;
use rust_decimal::Decimal;

fn cvt_json_2_decimal(
    val: Option<&serde_json::Value>,
    default_zero: bool,
) -> Result<Decimal, String> {
    match val {
        Some(v) => {
            if v.is_string() || v.is_number() {
                let val_str = v.to_string();
                return Decimal::from_str(&val_str).map_err(|e| {
                    log::error!("Invalid decimal value: {}, e={:?}", val_str, e);
                    "Invalid decimal value".to_string()
                });
            }
            Err(format!(
                "Unsupported value type: type={:?}, value={:?}",
                v.type_id(),
                v
            ))
        }
        None => match default_zero {
            true => Ok(Decimal::new(0, 0)),
            false => Err("Value is null".to_string()),
        },
    }
}

pub fn sum(items: &Vec<Item>, metrics: &str) -> Result<Decimal, String> {
    let mut sum = Decimal::new(0, 0);
    for item in items {
        let v = cvt_json_2_decimal(item.metrics.get(metrics), true)?;
        sum = sum.saturating_add(v);
    }
    Ok(sum)
}
