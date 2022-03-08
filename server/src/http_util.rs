use anyhow::anyhow;
use std::borrow::Cow;

use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::Selector;
use url::{ParseError, Url};

pub const PARALLELISM: usize = 8;

const USER_AGENT:&str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.109 Safari/537.36";

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .build()
        .unwrap()
});

pub fn http_client() -> &'static Client {
    &*HTTP_CLIENT
}

pub fn s(selector: &str) -> Selector {
    Selector::parse(selector).unwrap()
}

pub fn find_host(url: &str) -> anyhow::Result<String> {
    let parsed = Url::parse(url)?;
    let host_name = parsed
        .host()
        .ok_or_else(|| anyhow!("Didn't find host name in the {url}"))?;
    Ok(format!("{}://{}", parsed.scheme(), host_name))
}

pub fn normalize_url<'a, 'b>(mut url: &'a str, host: &'b str) -> anyhow::Result<Cow<'a, str>> {
    match Url::parse(url) {
        Ok(_) => Ok(Cow::Borrowed(url)),
        Err(ParseError::RelativeUrlWithoutBase) => {
            if url.starts_with('/') {
                url = &url[1..];
            }
            let host_url = Url::parse(host)?;
            let host_name = host_url
                .host()
                .ok_or_else(|| anyhow!("Failed to parse {host}"))?;
            Ok(Cow::Owned(format!(
                "{}://{}/{}",
                host_url.scheme(),
                host_name,
                url
            )))
        }
        _ => Err(anyhow!("Couldn't parse {url}")),
    }
}

#[cfg(test)]
mod test {
    use super::normalize_url;

    #[test]
    fn test_url_parser() {
        dbg!(normalize_url(
            "category/star-plus/star-plus-awards-concerts/",
            "https://www.desitellybox.me/",
        ))
        .unwrap();
        dbg!(normalize_url(
            "/category/star-plus/star-plus-awards-concerts/",
            "https://www.desitellybox.me/",
        ))
        .unwrap();
    }
}
