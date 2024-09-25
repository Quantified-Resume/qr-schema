use crate::service::err::sys_busy;

use super::super::err::cvt_err;
use qr_model::{Bucket, Builtin};
use qr_repo::{
    delete_bucket, delete_item_by_bucket_id, exist_item_by_bucket_id, insert_bucket, next_seq,
    select_bucket, select_bucket_by_builtin, Sequence,
};
use rusqlite::{Connection, TransactionBehavior};
use serde::Deserialize;

pub fn create_bucket(conn: &Connection, bucket: &mut Bucket) -> Result<i64, String> {
    // 1. check bucket
    check_builtin(conn, bucket)?;
    let seq = next_seq(conn, Sequence::Bucket).map_err(|e| cvt_err(e, "Failed to get sequence"))?;
    bucket.no = Some(seq);
    insert_bucket(conn, &bucket).map_err(|e| cvt_err(e, "Failed to save bucket"))
}

pub fn create_builtin_bucket(
    conn: &Connection,
    builtin: &Builtin,
    ref_id: Option<String>,
) -> Result<Bucket, String> {
    let mut bucket = Bucket::default_builtin(builtin, ref_id);
    let id = create_bucket(conn, &mut bucket)?;
    select_bucket_inner(conn, id)?.ok_or("Failed to create builtin bucket".to_string())
}

fn check_builtin(conn: &Connection, bucket: &Bucket) -> Result<(), String> {
    let Bucket {
        builtin,
        builtin_ref_id,
        ..
    } = bucket;
    if builtin.is_none() {
        return Ok(());
    }
    let b = match builtin {
        Some(b) => b,
        None => return Ok(()),
    };

    let exist_one = select_bucket_by_builtin(conn, b, builtin_ref_id.clone()).map_err(|e| {
        log::error!(
            "Errored fo select bucket by builtin: e={}, b={}, ref_id={:?}",
            e,
            b,
            builtin_ref_id.clone()
        );
        "Errored to query bucket".to_string()
    })?;
    match b.is_multiple() {
        true => match builtin_ref_id {
            None => Err("Ref ID is required for this builtin".to_string()),
            Some(_) => match exist_one {
                Some(v) => Err(format!("This bucket already exists: no={}", v.no.unwrap())),
                None => Ok(()),
            },
        },
        false => match exist_one {
            Some(v) => Err(format!("This bucket already exists: no={}", v.no.unwrap())),
            None => Ok(()),
        },
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BucketKey {
    pub id: Option<i64>,
    pub builtin: Option<Builtin>,
    pub builtin_ref_id: Option<String>,
}

impl BucketKey {
    pub fn new_from_id(id: i64) -> Self {
        BucketKey {
            id: Some(id),
            builtin: None,
            builtin_ref_id: None,
        }
    }
}

pub fn list_series_by_bucket_id(conn: &Connection, id: i64) -> Result<(), String> {
    let _bucket = select_bucket_inner(conn, id)?.ok_or("Bucket not found".to_string())?;
    // TODO
    Ok(())
}

fn select_bucket_inner(conn: &Connection, id: i64) -> Result<Option<Bucket>, String> {
    select_bucket(&conn, id).map_err(|e| {
        log::error!("Errored to query bucket: e={}, id={}", e, id);
        "Errored to query bucket".to_string()
    })
}

pub fn remove_bucket(conn: &mut Connection, id: i64, force: bool) -> Result<(), String> {
    check_bucket_exist(&conn, id)?;

    let exist_items = exist_item_by_bucket_id(&conn, id).map_err(|e| {
        log::error!("Failed to find bucket by id: err={}, id={}", e, id);
        "Failed to find bucket by id".to_string()
    })?;
    if exist_items && !force {
        return Err("Can't delete bucket with items".to_string());
    }
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sys_busy)?;
    let res = delete_item_by_bucket_id(&tx, id).and_then(|cnt| {
        log::info!("Deleted {} items of bucket: id={}", cnt, id);
        delete_bucket(&tx, id)
    });
    match res {
        Ok(_) => {
            log::info!("Deleted bucket: id={}", id);
            tx.commit().map_err(sys_busy)
        }
        Err(e) => tx.rollback().and_then(|_| Err(e)).map_err(sys_busy),
    }
}

pub fn check_bucket_exist(conn: &Connection, id: i64) -> Result<Bucket, String> {
    select_bucket_inner(conn, id)?.ok_or("Bucket not found".to_string())
}
