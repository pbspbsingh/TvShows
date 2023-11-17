use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt, TryStreamExt};
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

    let mut episode_error = None;
    for Episode { provider, links } in episode_parts {
        let parts_num = links.len();
        let metadata_result = stream::iter(links)
            .map(|(title, link)| async move {
                provider
                    .fetch_metadata(&link)
                    .await
                    .with_context(|| format!("'{title}': {provider:?} => {link}"))
                    .map(|meta_url| (title, meta_url))
            })
            .buffered(parts_num)
            .try_collect::<Vec<_>>()
            .await;
        match metadata_result {
            Ok(result) => {
                info!(
                    "Successfully loaded episode parts via '{:?}' in {:?}",
                    provider,
                    start.elapsed()
                );
                return Ok(Json(result));
            }
            Err(e) => {
                warn!("Failed to load episode parts: {e:?}");
                episode_error = Some(e);
            }
        }
    }
    error!(
        "Time taken for failed attempt to load parts: {:?}",
        start.elapsed()
    );
    let error = episode_error
        .map(|e| anyhow!("Failed to load {tv_channel} > {tv_show} > {episode}: {e:?}"))
        .unwrap_or_else(|| anyhow!("Failed to load {tv_channel} > {tv_show} > {episode}"));
    Err(error.into())
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
