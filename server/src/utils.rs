use std::time::SystemTime;

use chrono::{Datelike, Duration, Local, NaiveDate};

pub const CACHE_FOLDER: &str = "cache";

pub fn expiry_time() -> SystemTime {
    let now = Local::now().naive_local();
    let expiry_time = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(0, 30, 0);
    let expiry_time = expiry_time + Duration::days(1);
    let diff = expiry_time - now;
    SystemTime::now() + diff.to_std().unwrap()
}

#[cfg(test)]
mod test {
    use super::expiry_time;

    #[test]
    fn test_expiry() {
        println!("{:?}", expiry_time());
    }
}
