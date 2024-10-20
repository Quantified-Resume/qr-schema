use rust_decimal::Decimal;

pub fn json_val_to_sql_val(json: &serde_json::Value) -> rusqlite::types::Value {
    if json.is_null() {
        return rusqlite::types::Value::Null;
    } else if json.is_boolean() {
        let v = json
            .as_bool()
            .map(|v| match v {
                true => 1,
                false => 0,
            })
            .unwrap_or(0);
        return rusqlite::types::Value::Integer(v);
    } else if json.is_i64() {
        let v = json.as_i64().unwrap_or(0);
        return rusqlite::types::Value::Integer(v);
    } else if json.is_u64() {
        let v = match json.as_u64() {
            Some(v) => v,
            None => 0,
        };
        return rusqlite::types::Value::Integer(v as i64);
    } else {
        return rusqlite::types::Value::Text(json.to_string());
    }
}

pub fn json_val_to_decimal(json: &serde_json::Value) -> Result<Decimal, String> {
    if json.is_null() {
        return Ok(Decimal::ZERO);
    } else if json.is_boolean() {
        let v = match json.as_bool().unwrap_or(false) {
            true => Decimal::ONE,
            false => Decimal::ZERO,
        };
        return Ok(v);
    } else {
        todo!()
    }
}
