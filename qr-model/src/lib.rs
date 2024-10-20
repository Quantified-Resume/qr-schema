mod bucket;
mod builtin;
mod common;
mod item;
mod metrics;
mod tag;

pub use self::bucket::{Bucket, BucketStatus};
pub use self::builtin::Builtin;
pub use self::common::MetaItem;
pub use self::item::{Item, NamingEntity};
pub use self::metrics::{Metrics, MetricsValueType};

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
