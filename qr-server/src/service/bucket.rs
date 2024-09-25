use qr_model::{Bucket, Builtin};
use qr_repo::{insert_bucket, next_seq, select_bucket, select_bucket_by_builtin, Sequence};
use rusqlite::Connection;
use serde::Deserialize;

use super::super::err::{cvt_err, err};

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
    match create_bucket(conn, &mut bucket) {
        Ok(v) => select_bucket(conn, v).ok_or("Failed to create builtin bucket".to_string()),
        Err(e) => err(e, "Failed to create builtin bucket"),
    }
}

fn check_builtin(conn: &Connection, bucket: &Bucket) -> Result<(), String> {
    let builtin = match &bucket.builtin {
        None => return Ok(()),
        Some(b) => b,
    };

    let ref_id = bucket.builtin_ref_id.clone();
    let exist_one = select_bucket_by_builtin(conn, &builtin, ref_id.clone());
    match builtin.is_multiple() {
        true => match ref_id {
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
    let bucket = select_bucket(&conn, id).ok_or("Bucket not found".to_string())?;

    Ok(())
}
