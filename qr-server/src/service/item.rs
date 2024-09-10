use qr_model::{Bucket, BucketStatus, Builtin, Item};
use qr_repo::{insert_item, select_bucket, select_bucket_by_builtin, select_item_by_bid_and_rid};
use rusqlite::Connection;

use super::{bucket::create_builtin_bucket, check_metrics, BucketKey};

pub fn create_item(conn: &Connection, b_key: &BucketKey, item: &Item) -> Result<i64, String> {
    let mut item_clone = item.clone();
    // 1. check bucket
    let bid = match check_bucket(conn, b_key).and_then(|b| check_builtin(&b, &mut item_clone)) {
        Err(e) => return Err(e),
        Ok(val) => val,
    };
    // 3. check not exist
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

fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Bucket, String> {
    let bucket_opt = match key.id {
        Some(id) => select_bucket(conn, id),
        None => match &key.builtin {
            None => return Err("No bucket specified".to_string()),
            Some(val) => get_or_create_builtin(conn, val.clone(), key.builtin_ref_id.clone()),
        },
    };

    match bucket_opt {
        None => Err("Bucket not found".to_string()),
        Some(val) => match BucketStatus::Enabled == val.status {
            true => Ok(val),
            false => Err("Invalid bucket status".to_string()),
        },
    }
}

fn get_or_create_builtin(
    conn: &Connection,
    builtin: Builtin,
    ref_id: Option<String>,
) -> Option<Bucket> {
    match select_bucket_by_builtin(conn, &builtin, ref_id.clone()) {
        Some(v) => Some(v),
        None => match create_builtin_bucket(conn, builtin, ref_id) {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("Failed to get_or_create_builtin: {}", e);
                None
            }
        },
    }
}

pub fn check_builtin(bucket: &Bucket, item: &mut Item) -> Result<i64, String> {
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
