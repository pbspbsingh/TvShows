use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::header;
use axum::http::{HeaderMap, Request};
use tracing::debug;

use crate::error::HttpError;

const ASSET_DIR: &str = "dist";

pub async fn static_assets(req: Request<Body>) -> Result<(HeaderMap, Vec<u8>), HttpError> {
    async fn read_file(path: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
        let path = path.as_ref();
        debug!("Loading: {:?}", path.canonicalize()?);
        Ok(tokio::fs::read(path).await?)
    }

    let path = if req.uri().path() == "/" {
        "index.html"
    } else {
        &req.uri().path()[1..]
    };

    let mut path = PathBuf::from(ASSET_DIR).join(path);
    if !path.exists() {
        path = PathBuf::from(ASSET_DIR).join("index.html");
    }

    let mut headers = HeaderMap::with_capacity(1);
    if let Some(mime) = mime_guess::from_path(&path).first() {
        headers.append(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
    }
    Ok((headers, read_file(path).await?))
}
