use std::fmt::Display;

pub fn err<T, E>(err: E, msg: &str) -> Result<T, String>
where
    E: Display,
{
    log::error!("{}: {}", msg, err);
    Err(String::from(msg))
}

pub fn cvt_err<E>(err: E, msg: &str) -> String
where
    E: Display,
{
    log::error!("{}: {}", msg, err);
    String::from(msg)
}

pub fn or_none<T, E>(result: Result<T, E>, msg: &str) -> Option<T>
where
    E: Display,
{
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            log::error!("{}: {}", msg, e);
            None
        }
    }
}

pub fn none<T, E>(err: E, msg: &str) -> Option<T>
where
    E: Display,
{
    log::error!("{}: {}", msg, err);
    None
}
