use anyhow::anyhow;
use reqwest::header;
use serde::Deserialize;
use tokio::time::Instant;
use tracing::*;

use crate::http_util::{find_host, http_client};
use crate::tv_episodes::providers::flash_player::find_source;

use super::find_iframe;

pub async fn find_m3u8(html: &str, referer: &str) -> anyhow::Result<(String, String)> {
    #[derive(Deserialize, Debug)]
    struct Source {
        src: String,
    }

    let start = Instant::now();
    let iframe_src = find_iframe(html, referer)?;
    debug!("Got iframe src: {iframe_src}");
    let html = http_client()
        .get(&iframe_src)
        .header(header::REFERER, find_host(referer)?)
        .send()
        .await?
        .text()
        .await?;
    let vid_src =
        find_source(&html).ok_or_else(|| anyhow!("Failed to find video source in {iframe_src}"))?;
    let vid_src = serde_json::from_str::<Source>(vid_src)?;
    info!(
        "Time taken to resolve DailyMotion: {}",
        start.elapsed().as_millis()
    );
    Ok((vid_src.src, iframe_src))
}
