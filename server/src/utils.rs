use std::time::{Duration as StdDuration, SystemTime};

use chrono::{Datelike, Duration, Local, NaiveDate};
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use tracing::*;

pub const TV_CHANNEL_FILE: &str = "channels.json";

pub const TV_SHOWS_FILE: &str = "tv_shows.json";

pub const EXPIRY: StdDuration = StdDuration::from_secs(2 * 24 * 60 * 60);

static CACHE_FOLDER: OnceCell<String> = OnceCell::new();

static WHITE_SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub fn set_cache_folder(folder: &str) -> anyhow::Result<()> {
    CACHE_FOLDER
        .set(folder.to_owned())
        .map_err(|_| anyhow::anyhow!("Failed to init the cache folder: {folder}"))?;
    info!("Using cache folder: {folder}");
    Ok(())
}

pub fn cache_folder() -> &'static str {
    CACHE_FOLDER.get().map(|s| s as &str).unwrap_or("cache")
}

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

pub fn fix_title(title: impl Into<String>) -> String {
    const REPLACE_STR: &[(&str, &str)] = &[("Watch Online", ""), ("&amp;", "&")];
    const STRIP_SUFFIX: &[&str] = &["–", "-", "Shows"];

    let mut title = title.into();
    for &(old, new) in REPLACE_STR {
        title = title.replace(old, new);
    }
    let mut title = title.trim();
    for &strip_suff in STRIP_SUFFIX {
        title = title.strip_suffix(strip_suff).unwrap_or(title).trim();
    }
    WHITE_SPACE_REGEX.replace_all(title, " ").into_owned()
}

#[cfg(test)]
mod test {
    use super::expiry_time;
    use super::fix_title;

    #[test]
    fn test_expiry() {
        println!("{:?}", expiry_time());
    }

    #[test]
    fn test_title() {
        println!("{}", fix_title("Whos Your Daddy Watch Online – Episode 12"));
        println!("{}", fix_title("Whos Your Daddy   Watch Online – "));
        println!("{}", fix_title("&amp; TV Shows"));
    }
}
