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
use tokio::sync::mpsc::{channel, Receiver};
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
    debug!("Status: {}, header: {:#?}", res.status(), res.headers());
    Ok(response_to_body(res, is_mp4).await?)
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
