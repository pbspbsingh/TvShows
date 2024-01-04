use crate::response::Response;
use dashmap::DashMap;
use hyper::client::connect::dns::Name;
use hyper::header;
use log::*;
use reqwest::Client;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::time::{Duration, Instant};

pub struct ResolverInner {
    client: Client,
    cache: DashMap<Name, CachedNames>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
pub enum DnsType {
    A,
    AAAA,
}

impl Display for DnsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dns_type = match self {
            DnsType::A => "A",
            DnsType::AAAA => "AAAA",
        };
        write!(f, "{dns_type}")
    }
}

#[derive(Debug)]
struct CachedNames {
    expires_at: Instant,
    addrs: Vec<IpAddr>,
}

impl ResolverInner {
    pub fn new() -> Self {
        let duration = Duration::from_secs(5);
        let client = Client::builder()
            .connect_timeout(duration)
            .timeout(duration)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0")
            .build()
            .expect("Failed to build http client for cloudflared");
        let cache = DashMap::new();
        Self { client, cache }
    }

    pub async fn resolve_ips(&self, name: Name, dns_type: DnsType) -> Result<Vec<IpAddr>, String> {
        if let Some(cached) = self.cache.get(&name) {
            if cached.expires_at >= Instant::now() {
                let addrs = cached.addrs.clone();
                debug!("Cache hit: {name} => {addrs:?}");
                return Ok(addrs);
            }
        }

        let start = Instant::now();
        let doh_url = format!("https://cloudflare-dns.com/dns-query?name={name}&type={dns_type}");
        debug!("DNS fetch url {name} => {doh_url}");
        let request = self
            .client
            .get(&doh_url)
            .header(header::ACCEPT, "application/dns-json")
            .send();
        debug!("Sent the dns request for {name}");
        let response = request
            .await
            .map_err(|e| format!("DNS request failed {name}: {e:?}"))?
            .json::<Response>()
            .await
            .map_err(|e| format!("Parsing dns response failed for {name}: {e:?}"))?;
        debug!("Got the response for {name}");
        let (addrs, ttl) = response.resolve();

        let elapsed = start.elapsed();
        info!("DNS {name} => {addrs:?}, expires after {ttl}s, fetched in {elapsed:?}");

        if addrs.is_empty() {
            return Err(format!("Failed to fetch dns for {name}"));
        }

        let cached_names = CachedNames {
            expires_at: Instant::now() + Duration::from_secs(ttl.max(300) as u64),
            addrs: addrs.clone(),
        };
        self.cache.insert(name, cached_names);

        Ok(addrs)
    }
}

#[cfg(test)]
mod tests {
    use super::{DnsType, ResolverInner};
    use hyper::client::connect::dns::Name;
    use std::str::FromStr;

    #[tokio::test]
    async fn resolve_multiple() {
        let resolver = ResolverInner::new();
        let resolved = tokio::try_join!(
            resolver.resolve_ips(Name::from_str("www.amazon.com").unwrap(), DnsType::AAAA),
            resolver.resolve_ips(Name::from_str("www.facebook.com").unwrap(), DnsType::AAAA),
            resolver.resolve_ips(Name::from_str("www.google.com").unwrap(), DnsType::AAAA),
            resolver.resolve_ips(Name::from_str("www.amazon.com").unwrap(), DnsType::A),
            resolver.resolve_ips(Name::from_str("www.facebook.com").unwrap(), DnsType::A),
            resolver.resolve_ips(Name::from_str("www.google.com").unwrap(), DnsType::AAAA),
        );
        println!("{resolved:#?}");
    }
}
