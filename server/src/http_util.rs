use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use cloudflare_resolver::CloudflareResolver;
use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::Selector;
use url::{ParseError, Url};

pub const PARALLELISM: usize = 8;

pub const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/119.0";

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .cookie_store(true)
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        .dns_resolver(Arc::new(CloudflareResolver::new()))
        .connect_timeout(Duration::from_secs(60))
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

pub fn normalize_url<'a, 'b>(url: &'a str, host: &'b str) -> anyhow::Result<Cow<'a, str>> {
    match Url::parse(url) {
        Ok(_) => Ok(Cow::Borrowed(url)),
        Err(ParseError::RelativeUrlWithoutBase) => {
            let host_url = Url::parse(host)?;
            Ok(Cow::Owned(if let Some(url) = url.strip_prefix("//") {
                format!("{}://{}", host_url.scheme(), url)
            } else if let Some(url) = url.strip_prefix('/') {
                let host_name = host_url
                    .host()
                    .ok_or_else(|| anyhow!("Failed to parse {host}"))?;
                format!("{}://{}/{}", host_url.scheme(), host_name, url)
            } else {
                let host = host.rfind('/').map(|idx| &host[..idx]).unwrap_or(host);
                format!("{}/{}", host, url)
            }))
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
            "https://www.desitellybox.me/loda/lahsun",
        ))
        .unwrap();
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
