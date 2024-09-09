use qr_model::{Bucket, Item};
use qr_repo::{insert_item, select_item_by_bid_and_rid};
use rusqlite::Connection;

use super::check_metrics;

pub fn create_item(conn: &Connection, bucket: &Bucket, item: &Item) -> Result<i64, String> {
    let mut item_clone = item.clone();
    // 1. check builtin
    let bid = match check_builtin(&bucket, &mut item_clone) {
        Err(e) => return Err(e),
        Ok(bid) => bid,
    };
    // 2. check not exist
    match select_item_by_bid_and_rid(conn, bid, &item.ref_id) {
        Err(e) => {
            log::error!("Failed to query exist one: {}", e);
            return Err("Internal error".to_string());
        }
        Ok(item_opt) => match item_opt {
            Some(_) => return Err("Ref ID duplicated".to_string()),
            None => {}
        },
    };
    // 3. insert
    insert_item(conn, bid, &item_clone).map_err(|e| {
        log::error!("Failed to create item: {}", e);
        "Failed to create item".to_string()
    })
}

fn check_builtin(bucket: &Bucket, item: &mut Item) -> Result<i64, String> {
    let bid = match bucket.id {
        None => return Err("Invalid bucket".to_string()),
        Some(v) => v,
    };
    let metrics = match &bucket.builtin {
        // TODO: check customized bucket
        None => return Ok(bid),
        Some(b) => check_metrics(b.get_metrics_def(), item.metrics.clone()),
    };
    match metrics {
        Err(e) => return Err(e),
        Ok(valid_val) => {
            item.metrics = valid_val;
        }
    };
    Ok(bid)
}
