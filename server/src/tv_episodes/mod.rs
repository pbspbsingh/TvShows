use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt};
use tracing::*;

use crate::error::HttpError;
use crate::models::Episode;
use crate::tv_shows::get_episode_parts;
use crate::utils::cache_folder;

mod metadata;
mod providers;

pub async fn episode_parts(
    Path(params): Path<HashMap<String, String>>,
) -> Result<impl IntoResponse, HttpError> {
    let start = Instant::now();
    let tv_channel = params
        .get("tv_channel")
        .ok_or_else(|| anyhow!("No tv channel"))?;
    let tv_show = params.get("tv_show").ok_or_else(|| anyhow!("No tv show"))?;
    let episode = params.get("episode").ok_or_else(|| anyhow!("No episode"))?;
    info!("Loading parts for {tv_channel} > {tv_show} > {episode}");
    let episode_parts = get_episode_parts(tv_channel, tv_show, episode)
        .await
        .ok_or_else(|| {
            anyhow!("Couldn't find TvEpisodes with {tv_channel} > {tv_show} > {episode}")
        })?;
    let n = episode_parts.len();
    let mut parts_stream = stream::iter(episode_parts)
        .map(|Episode { provider, links }| async move {
            let mut result = Vec::with_capacity(links.len());
            for (title, link) in links {
                let metadata = provider
                    .fetch_metadata(&link)
                    .await
                    .with_context(|| format!("'{title}': {provider:?} => {link}"))?;
                result.push((title, metadata));
            }
            let title = result
                .first()
                .map(|(title, _)| title.as_str())
                .unwrap_or("NA");
            info!("Successfully downloaded '{title}' via '{provider:?}'");
            Ok::<_, anyhow::Error>(result)
        })
        .buffer_unordered(n);
    while let Some(result) = parts_stream.next().await {
        match result {
            Err(e) => {
                warn!("Resolving metadata failed: {e:?}");
            }
            Ok(result) => {
                info!(
                    "Time taken to load parts: {}ms",
                    start.elapsed().as_millis()
                );
                return Ok(Json(result));
            }
        };
    }
    error!(
        "Time taken for failed attempt to load parts: {}ms",
        start.elapsed().as_millis()
    );
    Err(anyhow!("Failed to load {tv_channel} > {tv_show} > {episode}").into())
}

pub async fn get_metadata(
    Path(params): Path<HashMap<String, String>>,
) -> Result<impl IntoResponse, HttpError> {
    let folder = params
        .get("folder")
        .ok_or_else(|| anyhow!("Folder not present in url"))?;
    let file_name = params
        .get("file_name")
        .ok_or_else(|| anyhow!("File name not present in url"))?;
    let file = PathBuf::from(cache_folder()).join(folder).join(file_name);
    info!("Reading metadata from {file:?}");
    Ok(fs::read_to_string(file).map_err(anyhow::Error::from)?)
}
