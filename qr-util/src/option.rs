pub fn if_present<T, F>(opt: Option<T>, f: F)
where
    F: FnOnce(T) -> (),
{
    if opt.is_some() {
        f(opt.unwrap());
    }
}
