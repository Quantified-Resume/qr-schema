mod bucket;
mod convert;
mod core;
mod initializer;
mod item;
mod sequence;
mod util;

pub use self::bucket::{
    delete_bucket, insert_bucket, select_all_buckets, select_bucket, select_bucket_by_builtin,
    update_bucket,
};
pub use self::item::{delete_item_by_bucket_id, exist_item_by_bucket_id, insert_item};

pub use self::initializer::init_tables;
pub use self::sequence::{next_seq, Sequence};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
