mod bucket;
mod builtin;
mod item;
mod tag;

pub use self::bucket::{Bucket, BucketStatus};
pub use self::item::{Item, ItemType};
pub use self::builtin::Builtin;

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
