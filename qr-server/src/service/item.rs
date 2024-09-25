use std::vec;

use qr_model::{Bucket, BucketStatus, Builtin, Item};
use qr_repo::{
    insert_item, select_all_items, select_bucket, select_bucket_by_builtin,
    select_item_by_bid_and_rid,
};
use rusqlite::Connection;

use crate::err::cvt_err;

use super::{bucket::create_builtin_bucket, check_metrics, BucketKey};

pub fn create_item(conn: &Connection, b_key: &BucketKey, item: &Item) -> Result<i64, String> {
    // 1. check bucket
    let bucket = check_bucket(conn, b_key)?;
    // 2. create
    create_item_inner(conn, &bucket, &mut item.clone(), false)
}

fn create_item_inner(
    conn: &Connection,
    bucket: &Bucket,
    item: &mut Item,
    ignore_exist: bool,
) -> Result<i64, String> {
    // 1. check builtin
    let bid = check_builtin(bucket, item)?;
    // 2. check exist
    let item_opt = select_item_by_bid_and_rid(conn, bid, &item.ref_id)
        .map_err(|e| cvt_err(e, "Failed to query exist item"))?;
    if item_opt.is_some() {
        let exist = item_opt.unwrap();
        match ignore_exist {
            true => return Ok(exist.id.unwrap()),
            false => {
                log::error!(
                    "RefID duplicated while creating item: refId={}, bucketId={}",
                    item.ref_id,
                    bid
                );
                return Err("Ref ID duplicated".to_string());
            }
        }
    }

    // 3. insert
    insert_item(conn, bid, &item).map_err(|e| {
        log::error!("Failed to create item: {}", e);
        "Failed to create item".to_string()
    })
}

/// Batch create item
pub fn batch_create_item(conn: &mut Connection, bid: i64, items: Vec<Item>) -> Result<i64, String> {
    let bucket = check_bucket(conn, &BucketKey::new_from_id(bid))?;
    let tx = conn.transaction().map_err(|e| {
        log::error!("Failed to gain transaction: {}", e);
        "System is busy".to_string()
    })?;
    for item in &items {
        let res = create_item_inner(&tx, &bucket, &mut item.clone(), true);
        if !res.is_err() {
            let e = res.unwrap_err();
            match tx.rollback() {
                Ok(_) => {}
                Err(txe) => log::error!("Failed rollback: {}", txe),
            }
            return Err(e);
        }
    }
    match tx.commit() {
        Ok(_) => Ok(items.len() as i64),
        Err(e) => {
            log::error!("Failed to commit: {}", e);
            Err("System is busy".to_string())
        }
    }
}

fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Bucket, String> {
    let bucket_opt = match key.id {
        Some(id) => select_bucket(conn, id),
        None => match &key.builtin {
            None => return Err("No bucket specified".to_string()),
            Some(val) => get_or_create_builtin(conn, val.clone(), key.builtin_ref_id.clone()),
        },
    };
    let bucket = bucket_opt.ok_or("Bucket not found".to_string())?;
    if BucketStatus::Enabled != bucket.status {
        Ok(bucket)
    } else {
        Err("Invalid bucket status".to_string())
    }
}

fn get_or_create_builtin(
    conn: &Connection,
    builtin: Builtin,
    ref_id: Option<String>,
) -> Option<Bucket> {
    let exist = select_bucket_by_builtin(conn, &builtin, ref_id.clone());
    if exist.is_some() {
        return exist;
    }
    match create_builtin_bucket(conn, &builtin, ref_id) {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!(
                "Failed to get_or_create_builtin: builtin={}, err={}",
                builtin,
                e
            );
            None
        }
    }
}

pub fn check_builtin(bucket: &Bucket, item: &mut Item) -> Result<i64, String> {
    let Bucket { id, builtin, .. } = bucket;
    let bid = id.ok_or("Invalid bucket".to_string())?;
    item.metrics = match &builtin {
        // TODO: check customized bucket
        None => return Ok(bid),
        Some(b) => check_metrics(b.get_metrics_def(), item.metrics.clone()),
    }?;
    Ok(bid)
}

pub fn list_item_by_bucket_id(conn: &Connection, bid: i64) -> Result<Vec<Item>, String> {
    select_all_items(
        conn,
        vec![format!("bucket_id = ?1")],
        vec![rusqlite::types::Value::Integer(bid)],
    )
    .map_err(|e| {
        log::error!(
            "Error occurred when listing items of bucket: {:?}, bid={}",
            e,
            bid
        );
        "Failed to query items".to_string()
    })
}
