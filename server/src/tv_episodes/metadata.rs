use std::path::{Path, PathBuf};

use anyhow::anyhow;
use reqwest::header;
use tokio::fs;
use tracing::*;

use crate::http_util::{curl_get, http_client, normalize_url};
use crate::models::VideoProvider;
use crate::tv_channels::DESI_TV;
use crate::tv_episodes::providers::{dailymotion, flash_player, tv_logy};
use crate::utils::{hash, CACHE_FOLDER};

const METADATA_FILE: &str = "metadata.m3u8";

impl VideoProvider {
    pub async fn fetch_metadata(&self, link: &str) -> anyhow::Result<String> {
        debug!("Loading metadata of {self:?}:{link}");
        let hsh = hash(link);
        let metadata_file = PathBuf::from(CACHE_FOLDER).join(&hsh).join(METADATA_FILE);
        if metadata_file.exists() {
            return metadata_url(&metadata_file);
        }
        debug!("{metadata_file:?} doesn't exist");
        let html = http_client()
            .get(link)
            .header(header::REFERER, DESI_TV)
            .send()
            .await?
            .text()
            .await?;
        let (m3u8_url, referer) = match self {
            VideoProvider::TVLogy => tv_logy::find_m3u8(&html, link).await?,
            VideoProvider::FlashPlayer => flash_player::find_m3u8(&html, link).await?,
            VideoProvider::DailyMotion => dailymotion::find_m3u8(&html, link).await?,
            VideoProvider::NetflixPlayer => dailymotion::find_m3u8(&html, link).await?,
            _ => return Err(anyhow!("Not implemented yet")),
        };
        info!("Found M3U8 url: {m3u8_url} with referer: {referer}");
        let m3u8_content = if let Ok(res) = http_client()
            .get(&m3u8_url)
            .header(header::REFERER, &referer)
            .send()
            .await
        {
            res.text().await?
        } else {
            curl_get(&m3u8_url, &referer).await?
        };
        let video_url = find_best_video_url(&m3u8_content, &m3u8_url)?;
        info!("Found video url: {video_url}");

        let m3u8_content = if let Ok(res) = http_client()
            .get(&video_url)
            .header(header::REFERER, &referer)
            .send()
            .await
        {
            res.text().await?
        } else {
            curl_get(&video_url, &referer).await?
        };

        let m3u8_content = convert_m3u8(&m3u8_content, &video_url, &hsh)?;
        fs::create_dir_all(metadata_file.parent().unwrap()).await?;
        fs::write(&metadata_file, m3u8_content).await?;

        metadata_url(&metadata_file)
    }
}

fn find_best_video_url(m3u8: &str, host_url: &str) -> anyhow::Result<String> {
    let mut itr = m3u8.split('\n').rev().peekable();
    while let Some(cur) = itr.next() {
        if let Some(next) = itr.peek() {
            if next.starts_with("#EXT-X-STREAM-INF") {
                return Ok(normalize_url(cur, host_url)?.into_owned());
            }
        }
    }
    Err(anyhow!("Couldn't parse M3U8 content: '{m3u8}'"))
}

fn convert_m3u8(m3u8: &str, host_url: &str, hash: &str) -> anyhow::Result<String> {
    let mut result = Vec::new();
    let mut itr = m3u8.split('\n');
    while let Some(line) = itr.next() {
        result.push(line.to_owned());
        if line.starts_with("#EXTINF") {
            let next_line = itr
                .next()
                .ok_or_else(|| anyhow!("Missing next line after 'EXTINF'"))?;
            let url = normalize_url(next_line, host_url)?;
            let url = form_urlencoded::byte_serialize(url.as_bytes()).collect::<String>();
            result.push(format!("/media?hash={hash}&url={url}"));
        }
    }
    Ok(result.join("\n"))
}

fn metadata_url(path: &Path) -> anyhow::Result<String> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("No parent for {path:?}"))?
        .file_name()
        .ok_or_else(|| anyhow!("No file name for {path:?}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("No file name for {path:?}"))?;
    Ok(format!(
        "/metadata/{}/{}",
        parent.to_string_lossy(),
        file_name.to_string_lossy()
    ))
}
