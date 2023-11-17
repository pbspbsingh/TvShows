use dashmap::DashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use hyper::client::connect::dns::Name;
use hyper::header;
use reqwest::dns::{Resolve, Resolving};
use reqwest::Client;

use crate::response::Response;

mod response;

pub struct CloudflareResolver(Arc<ResolverInner>);

impl CloudflareResolver {
    pub fn new() -> Self {
        CloudflareResolver(Arc::new(ResolverInner::new()))
    }
}

impl Resolve for CloudflareResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.0.clone();
        Box::pin(async move {
            match resolver.resolve_ips(&name, "AAAA").await {
                Ok(ips) => return Ok(box_it(ips)),
                Err(e) => {
                    log::warn!("Failed to resolve IPv6 for {name}: {e:?}");
                }
            }

            Ok(box_it(resolver.resolve_ips(&name, "A").await?))
        })
    }
}

fn box_it(itr: Vec<IpAddr>) -> Box<dyn Iterator<Item = SocketAddr> + Send> {
    Box::new(itr.into_iter().map(|addr| SocketAddr::new(addr, 80)))
}

struct ResolverInner {
    client: Client,
    cache: DashMap<Name, CachedNames>,
}

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
            .build()
            .expect("Failed to build http client");
        let cache = DashMap::new();
        Self { client, cache }
    }

    async fn resolve_ips(&self, name: &Name, dns_type: &str) -> Result<Vec<IpAddr>, String> {
        if let Some(cached) = self.cache.get(name) {
            if cached.expires_at >= Instant::now() {
                log::debug!("Cache hit: {name} => {:?}", cached.addrs);
                return Ok(cached.addrs.clone());
            } else {
                log::warn!("Dns entries for {name} expired");
            }
        }

        let start = Instant::now();
        let doh_url = format!("https://cloudflare-dns.com/dns-query?name={name}&type={dns_type}");
        let request = self
            .client
            .get(&doh_url)
            .header(header::ACCEPT, "application/dns-json")
            .send();
        let response = request
            .await
            .map_err(|e| format!("DNS request failed {name}: {e:?}"))?
            .json::<Response>()
            .await
            .map_err(|e| format!("Parsing dns response failed for {name}: {e:?}"))?;
        let (addrs, ttl) = response.resolve();
        log::info!(
            "DNS {name} => {addrs:?}, expires after {ttl}s, fetched in {}ms",
            start.elapsed().as_millis()
        );

        if addrs.is_empty() {
            return Err(format!("Failed to fetch dns for {name}").into());
        }

        self.cache.insert(
            name.clone(),
            CachedNames {
                expires_at: Instant::now() + Duration::from_secs(ttl.max(60) as u64),
                addrs: addrs.clone(),
            },
        );

        Ok(addrs)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hyper::client::connect::dns::Name;
    use reqwest::dns::Resolve;

    use super::CloudflareResolver;

    #[tokio::test]
    async fn it_works() {
        let resolver = CloudflareResolver::new();
        for socket in resolver
            .resolve(Name::from_str("www.amazon.com").unwrap())
            .await
            .unwrap()
        {
            print!("{socket:?}, ");
        }
        println!();
    }
}
