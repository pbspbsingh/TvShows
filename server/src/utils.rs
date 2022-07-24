use std::time::{Duration as StdDuration, SystemTime};

use chrono::{Datelike, Duration, Local, NaiveDate};

pub const CACHE_FOLDER: &str = "cache";

pub const TV_CHANNEL_FILE: &str = "channels.json";

pub const TV_SHOWS_FILE: &str = "tv_shows.json";

pub const EXPIRY: StdDuration = StdDuration::from_secs(2 * 24 * 60 * 60);

pub fn expiry_time() -> SystemTime {
    let now = Local::now().naive_local();
    let expiry_time = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(1, 30, 0);
    let expiry_time = expiry_time + Duration::days(1);
    let diff = expiry_time - now;
    SystemTime::now() + diff.to_std().unwrap()
}

pub fn hash(input: impl AsRef<[u8]>) -> String {
    let hash_val = seahash::hash(input.as_ref());
    format!("{:x}", hash_val)
}

pub fn encode_uri_component(input: impl AsRef<[u8]>) -> String {
    form_urlencoded::byte_serialize(input.as_ref()).collect()
}

#[cfg(test)]
mod test {
    use super::expiry_time;

    #[test]
    fn test_expiry() {
        println!("{:?}", expiry_time());
    }
}
