use std::vec;

use qr_model::{Bucket, BucketStatus, Builtin, Item};
use qr_repo::{
    insert_item, select_all_items, select_bucket, select_bucket_by_builtin,
    select_item_by_bid_and_rid,
};
use rusqlite::Connection;

use crate::err::cvt_err;

use super::{bucket::create_builtin_bucket, check_metrics, err::sys_busy, BucketKey};

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
        if !ignore_exist {
            log::error!(
                "RefID duplicated while creating item: refId={}, bucketId={}",
                item.ref_id,
                bid
            );
            return Err("Ref ID duplicated".to_string());
        }
        return exist.id.ok_or("Unexpected error".to_string());
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
    let tx = conn.transaction().map_err(sys_busy)?;
    for item in &items {
        log::debug!("Item: {:?}", item);
        log::debug!("Cloned item: {:?}", item.clone());
        let res = create_item_inner(&tx, &bucket, &mut item.clone(), true);
        if res.is_err() {
            return tx.rollback().map_err(sys_busy).and(res);
        }
    }
    tx.commit().map_err(sys_busy)?;
    Ok(items.len() as i64)
}

fn check_bucket(conn: &Connection, key: &BucketKey) -> Result<Bucket, String> {
    let BucketKey {
        id,
        builtin,
        builtin_ref_id,
        ..
    } = key;
    let bucket = match id {
        Some(v) => select_bucket(conn, *v)
            .map_err(|e| {
                log::error!("Errored to query bucket: e={}, id={}", e, v);
                "Errored to query bucket".to_string()
            })?
            .ok_or("Bucket not found".to_string()),
        None => {
            let b = builtin.clone().ok_or("No bucket specified".to_string())?;
            get_or_create_builtin(conn, b.clone(), builtin_ref_id.clone())
        }
    }?;

    if BucketStatus::Enabled == bucket.status {
        Ok(bucket)
    } else {
        Err("Invalid bucket status".to_string())
    }
}

fn get_or_create_builtin(
    conn: &Connection,
    builtin: Builtin,
    ref_id: Option<String>,
) -> Result<Bucket, String> {
    let exist = select_bucket_by_builtin(conn, &builtin, ref_id.clone()).map_err(|e| {
        log::error!(
            "Errored to select bucket by builtin:e={}, builtin={:?}, ref_id={:?}",
            e,
            builtin,
            ref_id.clone(),
        );
        "Errored to find buckets".to_string()
    })?;
    match exist {
        Some(v) => Ok(v),
        None => create_builtin_bucket(conn, &builtin, ref_id).map_err(|e| {
            log::error!(
                "Failed to get_or_create_builtin: builtin={}, err={}",
                builtin,
                e
            );
            "Errored to find buckets".to_string()
        }),
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
