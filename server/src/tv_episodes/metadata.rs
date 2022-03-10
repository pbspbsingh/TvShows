use std::path::PathBuf;

use anyhow::anyhow;
use reqwest::header;
use tokio::fs;
use tracing::*;

use crate::http_util::{http_client, normalize_url};
use crate::models::VideoProvider;
use crate::tv_channels::DESI_TV;
use crate::tv_episodes::tv_logy;
use crate::utils::{hash, CACHE_FOLDER};

const METADATA_FILE: &str = "metadata.m3u8";

impl VideoProvider {
    pub async fn fetch_metadata(&self, link: &str) -> anyhow::Result<PathBuf> {
        debug!("Loading metadata of {self:?}:{link}");
        let hsh = hash(link);
        let metadata_file = PathBuf::from(CACHE_FOLDER).join(&hsh).join(METADATA_FILE);
        if metadata_file.exists() {
            return Ok(metadata_file);
        }
        let html = http_client()
            .get(link)
            .header(header::REFERER, DESI_TV)
            .send()
            .await?
            .text()
            .await?;
        let (m3u8_url, referer) = match self {
            VideoProvider::TVLogy => tv_logy::find_m3u8(&html, link).await?,
            _ => return Err(anyhow!("Not implemented yet")),
        };
        info!("Found M3U8 url: {m3u8_url} with referer: {referer}");
        let m3u8_content = http_client()
            .get(&m3u8_url)
            .header(header::REFERER, &referer)
            .send()
            .await?
            .text()
            .await?;
        let video_url = find_best_video_url(&m3u8_content)?;
        info!("Found video url: {video_url}");

        let m3u8_content = http_client()
            .get(video_url)
            .header(header::REFERER, &referer)
            .send()
            .await?
            .text()
            .await?;
        let m3u8_content = convert_m3u8(&m3u8_content, &referer, &hsh)?;
        fs::create_dir_all(metadata_file.parent().unwrap()).await?;
        fs::write(&metadata_file, m3u8_content).await?;
        Ok(metadata_file)
    }
}

fn find_best_video_url(m3u8: &str) -> anyhow::Result<&str> {
    let mut itr = m3u8.split('\n').rev().peekable();
    while let Some(cur) = itr.next() {
        if let Some(next) = itr.peek() {
            if next.starts_with("#EXT-X-STREAM-INF") {
                return Ok(cur);
            }
        }
    }
    Err(anyhow!("Couldn't parse M3U8 content: {m3u8}"))
}

fn convert_m3u8(m3u8: &str, referer: &str, hash: &str) -> anyhow::Result<String> {
    let encoded_referer = form_urlencoded::byte_serialize(referer.as_bytes()).collect::<String>();
    let lines = m3u8.split('\n').collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        result.push(line.to_owned());
        if line.starts_with("#EXTINF") {
            let next = lines[i];
            i += 1;
            let url = normalize_url(next, referer)?;
            let url = form_urlencoded::byte_serialize(url.as_bytes()).collect::<String>();
            result.push(format!(
                "/video/hash={hash}&url={url}&referer={encoded_referer}"
            ));
        }
    }
    Ok(result.join("\n"))
}
