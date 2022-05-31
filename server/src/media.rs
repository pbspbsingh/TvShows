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
    info!("{}: {}", request.method(), url);

    let mut req = http_client().request(request.method().clone(), url);
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
        let download_speed = download_speed::SENDER.get();
        while let Ok(Some(bytes)) = response.chunk().await {
            download_speed.and_then(|s| s.send(bytes.len()).ok());
            if sender.send(bytes).await.is_err() {
                break;
            }
        }
    });
    Ok(http_res.body(Body::from(body_stream(receiver)))?)
}

fn body_stream(
    receiver: Receiver<Bytes>,
) -> Box<dyn Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync>>> + Send> {
    Box::new(stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|bytes| (Ok(bytes), receiver))
    }))
}

pub mod download_speed {
    use std::time::Duration;

    use once_cell::sync::OnceCell;
    use tokio::sync::mpsc::{self, UnboundedSender};
    use tokio::time::{self, Instant};
    use tracing::info;

    pub(super) static SENDER: OnceCell<UnboundedSender<usize>> = OnceCell::new();

    const WAIT_TIME: Duration = Duration::from_secs(5);

    pub async fn init() {
        let (sender, mut receiver) = mpsc::unbounded_channel::<usize>();
        SENDER
            .set(sender)
            .expect("Failed to initialize download speed sender");

        let (mut start, mut total_bytes) = (Instant::now(), 0);
        while let Some(bytes) = time::timeout_at(start + WAIT_TIME, receiver.recv())
            .await
            .unwrap_or(Some(0))
        {
            total_bytes += bytes;
            let elapsed = start.elapsed();
            if elapsed >= WAIT_TIME {
                if total_bytes > 0 {
                    info!("Download speed: {}", bytes_per_second(total_bytes, elapsed));
                }
                (start, total_bytes) = (Instant::now(), 0);
            }
        }
    }

    fn bytes_per_second(bytes_count: usize, elapsed: Duration) -> String {
        const KB: f64 = 1024.;
        const MB: f64 = KB * KB;

        let millis = elapsed.as_millis();
        let bytes_count = bytes_count as f64;
        let bps = (bytes_count * 1000.) / (millis as f64);

        let data = if bytes_count >= MB {
            format!("{:.2}MB", bytes_count / MB)
        } else if bytes_count >= KB {
            format!("{:.2}KB", bytes_count / KB)
        } else {
            format!("{:.2}B", bytes_count)
        };
        let data_rate = if bps >= MB {
            format!("{:.2}MB/s", bps / MB)
        } else if bps >= KB {
            format!("{:.2}KB/s", bps / KB)
        } else {
            format!("{:.2}B/s", bps)
        };
        format!("{data} @ {data_rate}")
    }

    #[cfg(test)]
    mod test {
        use super::bytes_per_second;
        use std::time::Duration;

        #[test]
        fn test_bps() {
            println!("{}", bytes_per_second(2049000, Duration::from_secs(1)));
            println!("{}", bytes_per_second(2000, Duration::from_millis(500)));
            println!(
                "{}",
                bytes_per_second(2 * 1024 * 1024, Duration::from_millis(500))
            );
            println!("{}", bytes_per_second(2, Duration::from_millis(200)));
        }
    }
}
