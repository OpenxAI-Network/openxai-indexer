use std::time::SystemTime;

pub fn get_time_i64() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Invalid system time (duration from unix epoch).")
        .as_secs()
        .try_into()
        .expect("Epoch time does not fit in i64")
}

pub fn get_time_u64() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Invalid system time (duration from unix epoch).")
        .as_secs()
}
