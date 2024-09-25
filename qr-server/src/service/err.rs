pub fn sys_busy(e: rusqlite::Error) -> String {
    log::error!("System busy: err={}", e);
    "System is busy".to_string()
}
