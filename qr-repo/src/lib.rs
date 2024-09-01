mod bucket;
mod convert;
mod core;
mod initializer;
mod sequence;
mod util;

pub use self::bucket::select_bucket_by_id;
pub use self::initializer::init_tables;

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
