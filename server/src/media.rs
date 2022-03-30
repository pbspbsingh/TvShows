use std::collections::{HashMap, HashSet};
use std::error::Error;

use anyhow::anyhow;
use axum::body::{Body, Bytes};
use axum::extract::Query;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use futures::{stream, Stream};
use once_cell::sync::Lazy;
use reqwest::header;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::time::Instant;
use tracing::*;

use crate::error::HttpError;
use crate::http_util::http_client;

const CHANNEL_BUFFER: usize = 32;

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
        header::VARY,
    ])
});

pub async fn media(
    request: Request<Body>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, HttpError> {
    debug!(
        "{} media, headers= {:?} query params= {:?}",
        request.method(),
        request.headers(),
        params,
    );
    let url = params
        .get("url")
        .ok_or_else(|| anyhow!("No url found in query params"))?;
    let is_mp4 = params.get("is_mp4").map(|m| m == "true").unwrap_or(false);
    info!("{}: {}", request.method(), url);

    let mut req = http_client().request(request.method().clone(), url);
    for (key, val) in request.headers() {
        if is_mp4 || ALLOWED_HEADERS.contains(key.as_str()) {
            req = req.header(key, val);
        }
    }
    let res = req
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch {url}, {e:?}"))?;
    debug!("Status: {}, header: {:?}", res.status(), res.headers());
    Ok(response_to_body(res, is_mp4).await?)
}

async fn response_to_body(
    mut response: reqwest::Response,
    is_mp4: bool,
) -> anyhow::Result<Response<Body>> {
    let mut http_res = Response::builder().status(response.status());
    for (key, val) in response.headers() {
        if is_mp4 || ALLOWED_HEADERS.contains(key.as_str()) {
            http_res = http_res.header(key, val);
        }
    }
    let (sender, receiver) = channel::<Option<Bytes>>(CHANNEL_BUFFER);
    tokio::spawn(async move {
        let (mut cur_start, mut cur_count) = (Instant::now(), 0);
        let (net_start, mut net_count) = (Instant::now(), 0);
        while let Ok(bytes) = response.chunk().await {
            let len = bytes.as_ref().map(|b| b.len()).unwrap_or(0);
            net_count += len;
            cur_count += len;
            if bytes.is_none() {
                debug!("Done reading bytes from remote server");
                break;
            }
            if let Err(e) = sender.try_send(bytes) {
                match e {
                    TrySendError::Full(bytes) => {
                        debug!(
                            "Buffer is full, current speed of reading: {}",
                            bytes_per_second(cur_count, cur_start.elapsed().as_millis())
                        );
                        if sender.send(bytes).await.is_ok() {
                            (cur_start, cur_count) = (Instant::now(), 0);
                        } else {
                            debug!("Failed to send bytes as Receiver is dropped");
                            break;
                        }
                    }
                    TrySendError::Closed(_) => {
                        debug!("Can't send bytes as Receiver is dropped");
                        break;
                    }
                }
            }
        }
        info!(
            "Read all the bytes, last batch speed: {}, net speed: {}",
            bytes_per_second(cur_count, cur_start.elapsed().as_millis()),
            bytes_per_second(net_count, net_start.elapsed().as_millis()),
        );
    });
    Ok(http_res.body(Body::from(body_stream(receiver)))?)
}

fn body_stream(
    receiver: Receiver<Option<Bytes>>,
) -> Box<dyn Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync>>> + Send> {
    Box::new(stream::unfold(receiver, |mut receiver| async move {
        if let Some(Some(bytes)) = receiver.recv().await {
            Some((Ok(bytes), receiver))
        } else {
            None
        }
    }))
}

fn bytes_per_second(bytes_count: usize, millis: u128) -> String {
    const KB: f64 = 1024.;
    const MB: f64 = KB * KB;

    let bps = (bytes_count as f64) * 1000. / (millis as f64);
    if bps >= MB {
        format!("{:.2} MB/s", bps / MB)
    } else if bps >= KB {
        format!("{:.2} KB/s", bps / KB)
    } else {
        format!("{:.2} B/s", bps)
    }
}

#[cfg(test)]
mod test {
    use super::bytes_per_second;

    #[test]
    fn test_bps() {
        println!("{}", bytes_per_second(2049000, 1000));
        println!("{}", bytes_per_second(2000, 500));
        println!("{}", bytes_per_second(2 * 1024 * 1024, 500));
        println!("{}", bytes_per_second(2, 200));
    }
}
