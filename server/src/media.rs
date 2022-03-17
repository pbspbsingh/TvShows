use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::thread;
use std::thread::JoinHandle;

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::extract::Query;
use axum::http::{HeaderMap, Request};
use axum::response::{IntoResponse, Response};
use curl::easy::{Easy, List, WriteError};
use futures::{stream, Stream};
use once_cell::sync::Lazy;
use reqwest::header;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::RwLock;
use tracing::*;
use url::Url;

use crate::error::HttpError;
use crate::http_util::http_client;

const CHANNEL_BUFFER: usize = 32;

static BAD_HOSTS: Lazy<RwLock<HashSet<String>>> = Lazy::new(|| RwLock::new(HashSet::new()));

static ALLOWED_HEADERS: Lazy<HashSet<header::HeaderName>> = Lazy::new(|| {
    HashSet::from([
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        header::ACCEPT,
        header::ACCEPT_CHARSET,
        header::ACCEPT_ENCODING,
        header::ACCEPT_RANGES,
        header::CACHE_CONTROL,
        header::CONNECTION,
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::COOKIE,
        header::DATE,
        header::EXPIRES,
        header::ETAG,
        header::LAST_MODIFIED,
        header::RANGE,
        header::USER_AGENT,
        header::VARY,
    ])
});

pub async fn media(
    request: Request<Body>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HttpError> {
    debug!(
        "{} media, headers= {:#?} query params= {:#?}",
        request.method(),
        request.headers(),
        params,
    );
    let url = params
        .get("url")
        .ok_or_else(|| anyhow!("No url found in query params"))?;
    let is_mp4 = params.get("is_mp4").map(|m| m == "true").unwrap_or(false);
    let host_name = Url::parse(url).map_err(|_| anyhow!("Url not valid: {url}"))?;
    let host_name = host_name
        .host_str()
        .ok_or_else(|| anyhow!("No host name in {url}"))?;
    if BAD_HOSTS.read().await.contains(host_name) {
        warn!("'{host_name}' a bad host, need to use curl");
        return Ok(response_to_body_via_curl(url.to_owned(), request.headers().to_owned()).await?);
    }

    let mut req = http_client().request(request.method().clone(), url);
    for (key, val) in request.headers() {
        if is_mp4 || ALLOWED_HEADERS.contains(key.as_str()) {
            req = req.header(key, val);
        }
    }
    if let Ok(res) = req.send().await {
        debug!("Status: {}, header: {:#?}", res.status(), res.headers());
        Ok(response_to_body(res, is_mp4).await?)
    } else {
        warn!("Need to use curl for {url}");
        BAD_HOSTS.write().await.insert(host_name.to_owned());
        Ok(response_to_body_via_curl(url.to_owned(), request.headers().to_owned()).await?)
    }
}

async fn response_to_body(
    mut res: reqwest::Response,
    is_mp4: bool,
) -> anyhow::Result<Response<Body>> {
    let mut http_res = Response::builder().status(res.status());
    for (key, val) in res.headers() {
        if is_mp4 || ALLOWED_HEADERS.contains(key.as_str()) {
            http_res = http_res.header(key, val);
        }
    }
    let (sender, receiver) = channel::<Option<Bytes>>(CHANNEL_BUFFER);
    tokio::spawn(async move {
        while let Ok(bytes) = res.chunk().await {
            if bytes.is_none() || sender.send(bytes).await.is_err() {
                debug!("Either all data is read, or received is dropped");
                break;
            }
        }
    });
    Ok(http_res.body(Body::from(body_stream(receiver)))?)
}

async fn response_to_body_via_curl(
    url: String,
    headers: HeaderMap,
) -> anyhow::Result<Response<Body>> {
    let (header_sender, mut header_receiver) = channel::<String>(CHANNEL_BUFFER);
    let (body_sender, body_receiver) = channel::<Option<Bytes>>(CHANNEL_BUFFER);

    let _join: JoinHandle<anyhow::Result<()>> = thread::spawn(move || {
        let mut easy = Easy::new();
        easy.ssl_verify_host(false)?;
        easy.ssl_verify_peer(false)?;
        easy.url(&url)?;

        let mut header_list = List::new();
        for (key, value) in headers.iter() {
            if ALLOWED_HEADERS.contains(key.as_str()) {
                header_list.append(&format!("{}: {}", key.as_str(), value.to_str()?))?;
            }
        }
        easy.http_headers(header_list)?;
        easy.header_function(move |header| {
            let header_str = String::from_utf8_lossy(header).into_owned();
            header_sender.blocking_send(header_str).is_ok()
        })?;
        let mut transfer = easy.transfer();
        transfer.write_function(move |data| {
            body_sender
                .blocking_send(if !data.is_empty() {
                    Some(Bytes::copy_from_slice(data))
                } else {
                    None
                })
                .map_err(|_| WriteError::Pause)?;
            Ok(data.len())
        })?;
        transfer.perform()?;
        Ok(())
    });

    let mut response = Response::builder();
    while let Some(header) = header_receiver.recv().await {
        if header.trim().is_empty() {
            break;
        }
        let mut header_itr = header.splitn(2, ':');
        let key = match header_itr.next() {
            Some(key) => key.trim(),
            None => continue,
        };
        let value = match header_itr.next() {
            Some(val) => val.trim(),
            None => continue,
        };
        if !(key.is_empty() || value.is_empty()) {
            response = response.header(key, value);
        }
    }
    Ok(response.body(Body::from(body_stream(body_receiver)))?)
}

fn body_stream(
    receiver: Receiver<Option<Bytes>>,
) -> Box<dyn Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync>>> + Send> {
    Box::new(stream::unfold(receiver, |mut receiver| async move {
        if let Some(Some(bytes)) = receiver.recv().await {
            return Some((Ok(bytes), receiver));
        }
        None
    }))
}
