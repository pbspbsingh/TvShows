use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::anyhow;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt};
use scraper::Html;
use tracing::*;

use crate::error::HttpError;
use crate::http_util::{normalize_url, s, PARALLELISM};
use crate::models::{Episode, VideoProvider};
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
    let parts = get_episode_parts(tv_channel, tv_show, episode)
        .await
        .ok_or_else(|| {
            anyhow!("Couldn't find TvEpisodes with {tv_channel} > {tv_show} > {episode}")
        })?;
    for Episode { provider, links } in parts {
        let parts_num = links.len();
        let result = stream::iter(links)
            .map(|(title, link)| async { (title, fetch_metadata(provider, link).await) })
            .buffered(PARALLELISM)
            .collect::<Vec<_>>()
            .await;
        let result = result
            .into_iter()
            .filter_map(|(title, url)| url.map(|path| (title, path)))
            .collect::<Vec<_>>();
        if result.len() == parts_num {
            info!(
                "Time taken to load parts: {}ms",
                start.elapsed().as_millis()
            );
            return Ok(Json(result));
        } else {
            warn!(
                "{:?} returned only {} parts while {} was expected",
                provider,
                result.len(),
                parts_num
            );
        }
    }
    info!(
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

pub fn find_iframe(html: &str, base_url: &str) -> anyhow::Result<String> {
    let doc = Html::parse_document(html);
    let url = doc
        .select(&s("iframe[allowfullscreen]"))
        .next()
        .and_then(|i| i.value().attr("src"))
        .ok_or_else(|| anyhow!("Failed to find iframe"))?;
    Ok(normalize_url(url, base_url)?.into_owned())
}

async fn fetch_metadata(provider: VideoProvider, link: String) -> Option<String> {
    match provider.fetch_metadata(&link).await {
        Ok(url) => Some(url),
        Err(e) => {
            error!("Couldn't fetch metadata for {link}: {e}");
            None
        }
    }
}
