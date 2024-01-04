use crate::resolver_inner::DnsType;
use default_net::Gateway;
use hyper::client::connect::dns::Name;
use log::*;
use reqwest::dns::{Resolve, Resolving};
use resolver_inner::ResolverInner;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

mod resolver_inner;
mod response;

pub struct CloudflareResolver {
    resolver: Arc<ResolverInner>,
    dns_type_order: [DnsType; 2],
}

impl CloudflareResolver {
    pub fn new() -> Self {
        CloudflareResolver {
            resolver: Arc::new(ResolverInner::new()),
            dns_type_order: dns_type_order(),
        }
    }
}

impl Resolve for CloudflareResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let dns_type_order = self.dns_type_order;
        let resolver = self.resolver.clone();
        Box::pin(async move {
            let mut err = None;
            for dns_type in dns_type_order {
                match resolver.resolve_ips(name.clone(), dns_type).await {
                    Ok(ips) => return Ok(box_it(ips)),
                    Err(e) => {
                        warn!("Failed to resolve DNS for {name}/{dns_type}: {e:?}");
                        err = Some(e);
                    }
                }
            }
            let default_err = || format!("DNS resolution error for {name}");
            Err(err.unwrap_or_else(default_err).into())
        })
    }
}

fn box_it(itr: Vec<IpAddr>) -> Box<dyn Iterator<Item = SocketAddr> + Send> {
    Box::new(itr.into_iter().map(|addr| SocketAddr::new(addr, 80)))
}

fn dns_type_order() -> [DnsType; 2] {
    match default_net::get_default_gateway() {
        Ok(Gateway { ip_addr, .. }) => {
            if matches!(ip_addr, IpAddr::V4(_)) {
                info!("Default gateway is IPv4.");
                return [DnsType::A, DnsType::AAAA];
            } else {
                info!("Default gateway is IPv6.");
            }
        }
        Err(e) => {
            warn!("Failed to get the default gateway: {e}");
        }
    };
    [DnsType::AAAA, DnsType::A]
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
