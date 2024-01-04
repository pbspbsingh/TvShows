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
use tokio::sync::mpsc::{self, Receiver};
use tracing::*;

use crate::error::HttpError;
use crate::http_util::http_client;

const CHANNEL_BUFFER: usize = 32;

static ALLOWED_HEADERS: Lazy<HashSet<String>> = Lazy::new(|| {
    [
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        header::ACCEPT,
        header::ACCEPT_CHARSET,
        header::ACCEPT_ENCODING,
        header::ACCEPT_RANGES,
        header::CACHE_CONTROL,
        header::CONTENT_TYPE,
        header::CONTENT_LENGTH,
        header::CONTENT_RANGE,
        header::COOKIE,
        header::DATE,
        header::EXPIRES,
        header::ETAG,
        header::LAST_MODIFIED,
        header::PRAGMA,
        header::RANGE,
        header::VARY,
    ]
    .into_iter()
    .map(|header| header.as_str().to_lowercase())
    .collect()
});

pub async fn media(
    Query(params): Query<HashMap<String, String>>,
    request: Request<Body>,
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
    let referer = params.get("referer");
    info!("{}: {} [Referer:{:?}]", request.method(), url, referer);

    let mut req = http_client().request(request.method().clone(), url);
    if let Some(referer) = referer {
        req = req.header(header::REFERER, referer);
    }
    for (key, val) in request.headers() {
        if ALLOWED_HEADERS.contains(&key.as_str().to_lowercase()) {
            req = req.header(key, val);
        }
    }
    let res = req
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch {url}, {e:?}"))?;
    debug!("Status: {}, header: {:?}", res.status(), res.headers());
    Ok(response_to_body(res).await?)
}

async fn response_to_body(mut response: reqwest::Response) -> anyhow::Result<Response<Body>> {
    let mut http_res = Response::builder().status(response.status());
    let mut ignored_headers = Vec::new();
    for (key, val) in response.headers() {
        if ALLOWED_HEADERS.contains(&key.as_str().to_lowercase()) {
            http_res = http_res.header(key, val);
        } else {
            ignored_headers.push((key, val));
        }
    }
    if !ignored_headers.is_empty() {
        debug!("Ignored headers: {ignored_headers:?}");
    }

    let (sender, receiver) = mpsc::channel(CHANNEL_BUFFER);
    tokio::spawn(async move {
        while let Ok(Some(bytes)) = response.chunk().await {
            if sender.send(bytes).await.is_err() {
                break;
            }
        }
    });
    Ok(http_res.body(body_stream(receiver).into())?)
}

fn body_stream(
    receiver: Receiver<Bytes>,
) -> Box<dyn Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync>>> + Send> {
    Box::new(stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|bytes| (Ok(bytes), receiver))
    }))
}
