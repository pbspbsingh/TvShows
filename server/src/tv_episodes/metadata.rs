use std::path::{Path, PathBuf};

use anyhow::anyhow;
use reqwest::header;
use tokio::fs;
use tracing::*;

use crate::http_util::{http_client, normalize_url};
use crate::models::VideoProvider;
use crate::tv_channels::DESI_TV;
use crate::tv_episodes::providers::{dailymotion, flash_player, speed, tv_logy};
use crate::utils::{cache_folder, encode_uri_component, hash};

const METADATA_FILE: &str = "metadata.m3u8";

impl VideoProvider {
    pub async fn fetch_metadata(&self, link: &str) -> anyhow::Result<String> {
        debug!("Loading metadata of {self:?}:{link}");
        let hsh = hash(link);
        let metadata_file = PathBuf::from(cache_folder()).join(&hsh).join(METADATA_FILE);
        if metadata_file.exists() {
            return if self.is_mp4() {
                Ok(fs::read_to_string(metadata_file).await?)
            } else {
                metadata_url(&metadata_file)
            };
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
            VideoProvider::DailyMotion | VideoProvider::NetflixPlayer => {
                dailymotion::find_m3u8(&html, link).await?
            }
            VideoProvider::Speed | VideoProvider::Vkprime => speed::find_mp4(&html, link).await?,
        };
        if self.is_mp4() {
            info!("Found mp4 url: {m3u8_url} with referer: {referer}");
            let url = encode_uri_component(m3u8_url);
            let url = format!("/media?is_mp4=true&hash={hsh}&url={url}");

            fs::create_dir_all(metadata_file.parent().unwrap()).await?;
            fs::write(metadata_file, &url).await?;

            Ok(url)
        } else {
            info!("Found M3U8 url: {m3u8_url} with referer: {referer}");
            let m3u8_content = http_client()
                .get(&m3u8_url)
                .header(header::REFERER, &referer)
                .send()
                .await?
                .text()
                .await?;
            let video_url = find_best_video_url(&m3u8_content, &m3u8_url)?;
            info!("Found video url: {video_url}");

            let m3u8_content = http_client()
                .get(&video_url)
                .header(header::REFERER, &referer)
                .send()
                .await?
                .text()
                .await?;
            let m3u8_content = convert_m3u8(&m3u8_content, &video_url, &hsh)?;
            fs::create_dir_all(metadata_file.parent().unwrap()).await?;
            fs::write(&metadata_file, m3u8_content).await?;

            metadata_url(&metadata_file)
        }
    }

    pub fn is_mp4(&self) -> bool {
        match self {
            VideoProvider::TVLogy => false,
            VideoProvider::FlashPlayer => false,
            VideoProvider::DailyMotion => false,
            VideoProvider::NetflixPlayer => false,
            VideoProvider::Speed => true,
            VideoProvider::Vkprime => true,
        }
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
            let url = encode_uri_component(&*url);
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
