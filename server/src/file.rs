use axum::body::Body;
use axum::http::header;
use axum::http::{HeaderMap, Request};

use crate::error::HttpError;

const DEFAULT_FILE: &str = "index.html";

pub async fn static_assets(req: Request<Body>) -> Result<(HeaderMap, Vec<u8>), HttpError> {
    let path = if req.uri().path() == "/" {
        DEFAULT_FILE
    } else {
        &req.uri().path()[1..]
    };

    let path = if assets::validate(path) {
        path
    } else {
        DEFAULT_FILE
    };
    let mut headers = HeaderMap::with_capacity(1);
    if let Some(mime) = mime_guess::from_path(path).first() {
        headers.append(header::CONTENT_TYPE, mime.as_ref().parse().unwrap());
    }
    Ok((headers, assets::read_file(path).await?))
}

#[cfg(debug_assertions)]
mod assets {
    use std::path::Path;

    use tokio::fs;

    const ASSET_DIR: &str = "dist";

    pub fn validate(path: &str) -> bool {
        Path::new(ASSET_DIR).join(path).exists()
    }

    pub async fn read_file(path: &str) -> anyhow::Result<Vec<u8>> {
        let content = fs::read(Path::new(ASSET_DIR).join(path)).await?;
        Ok(content)
    }
}

#[cfg(not(debug_assertions))]
mod assets {
    use include_dir::{include_dir, Dir};

    static STATIC_ASSETS: Dir<'_> = include_dir!("dist");

    pub fn validate(path: &str) -> bool {
        STATIC_ASSETS.contains(path)
    }

    pub async fn read_file(path: &str) -> anyhow::Result<Vec<u8>> {
        let file = STATIC_ASSETS
            .get_file(path)
            .ok_or(anyhow::anyhow!("Couldn't get file from STATIC_ASSETS"))?;
        Ok(file.contents().to_owned())
    }
}
