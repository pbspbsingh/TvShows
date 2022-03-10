use std::collections::HashMap;

use anyhow::anyhow;
use axum::extract::Path;
use axum::response::IntoResponse;
use tracing::*;

use crate::error::HttpError;
use crate::models::Episode;
use crate::tv_shows::get_episode_parts;

mod metadata;
mod tv_logy;

pub async fn episode_parts(
    Path(params): Path<HashMap<String, String>>,
) -> Result<impl IntoResponse, HttpError> {
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
        for (_, link) in &links {
            provider.fetch_metadata(link).await?;
        }
    }
    Ok("{}")
}
