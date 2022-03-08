use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use axum::body::Bytes;
use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt};
use once_cell::sync::OnceCell;
use reqwest::header;
use scraper::Html;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::*;

use crate::error::HttpError;
use crate::http_util::{http_client, normalize_url, s, PARALLELISM};
use crate::models::{TvChannel, TvSoap};
use crate::utils::CACHE_FOLDER;

const DESI_TV: &str = "https://www.desitellybox.me/";

const TV_CHANNEL_FILE: &str = "channels.json";

const EXPIRY_DURATION: Duration = Duration::from_secs(7 * 24 * 60 * 60); // A Week

static TV_CHANNEL_STATE: OnceCell<TvChannelState> = OnceCell::new();

struct TvChannelState {
    tv_channels: RwLock<Vec<TvChannel>>,
    expires_at: RwLock<SystemTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TvChannelResponse {
    title: String,
    icon: Option<String>,
    soaps: Vec<String>,
    completed: Vec<String>,
}

impl From<&TvChannel> for TvChannelResponse {
    fn from(tv: &TvChannel) -> Self {
        TvChannelResponse {
            title: tv.title.to_owned(),
            icon: tv.icon.to_owned(),
            soaps: tv.soaps.iter().map(|s| s.title.to_owned()).collect(),
            completed: tv
                .completed_soaps
                .iter()
                .map(|s| s.title.to_owned())
                .collect(),
        }
    }
}

pub async fn channel_home() -> Result<impl IntoResponse, HttpError> {
    async fn _channel_home() -> anyhow::Result<Json<Vec<TvChannelResponse>>> {
        let cache_file = PathBuf::from(CACHE_FOLDER).join(TV_CHANNEL_FILE);
        if TV_CHANNEL_STATE.get().is_none() {
            let state = if cache_file.exists() {
                info!("Loading channels state from {cache_file:?}");
                let content = fs::read_to_string(&cache_file).await?;
                let metadata = fs::metadata(&cache_file).await?;
                TvChannelState {
                    tv_channels: RwLock::new(serde_json::from_str(&content)?),
                    expires_at: RwLock::new(metadata.modified()? + EXPIRY_DURATION),
                }
            } else {
                warn!("Channel state file doesn't exist, initialing with dummies");
                TvChannelState {
                    tv_channels: RwLock::new(Vec::new()),
                    expires_at: RwLock::new(SystemTime::now() - EXPIRY_DURATION),
                }
            };
            TV_CHANNEL_STATE.set(state).ok();
        }
        let state = TV_CHANNEL_STATE
            .get()
            .ok_or_else(|| anyhow::anyhow!("Wtf, Channel state is not initialized"))?;
        if *state.expires_at.read().await <= SystemTime::now() {
            info!("TV channels list have expired, time to refresh it");

            let tv_channels = download_tv_channels().await?;
            fs::write(cache_file, serde_json::to_string_pretty(&tv_channels)?).await?;

            *state.expires_at.write().await = SystemTime::now() + EXPIRY_DURATION;
            *state.tv_channels.write().await = tv_channels;
        }
        Ok(Json(
            state
                .tv_channels
                .read()
                .await
                .iter()
                .map(TvChannelResponse::from)
                .collect(),
        ))
    }
    Ok(_channel_home().await?)
}

#[instrument]
async fn download_tv_channels() -> anyhow::Result<Vec<TvChannel>> {
    let start = Instant::now();
    info!("Loading TV channels from {DESI_TV}");

    let html = http_client().get(DESI_TV).send().await?.text().await?;
    let (mut tv_channels, completed_shows) = parse_channels(&html)?;

    let mut icons = stream::iter(tv_channels.clone())
        .map(|TvChannel { title, icon, .. }| async move { (title, download_icon(icon).await) })
        .buffered(PARALLELISM)
        .collect::<HashMap<_, _>>()
        .await;
    let mut completed_shows = stream::iter(completed_shows.into_iter())
        .map(|(title, url)| async move { (title, download_completed_shows(&url).await) })
        .buffered(PARALLELISM)
        .collect::<HashMap<_, _>>()
        .await;
    for tv_chn in &mut tv_channels {
        tv_chn.icon = if let Some(icon) = icons.remove(&tv_chn.title) {
            icon
        } else {
            None
        };
        if let Some(Ok(completed_soaps)) = completed_shows.remove(&tv_chn.title) {
            tv_chn.completed_soaps = completed_soaps;
        }
    }
    info!(
        "{} Channels loaded in {}ms",
        tv_channels.len(),
        start.elapsed().as_millis()
    );
    Ok(tv_channels)
}

fn parse_channels(html: &str) -> anyhow::Result<(Vec<TvChannel>, HashMap<String, String>)> {
    let mut tv_channels = Vec::new();
    let mut completed_shows = HashMap::new();
    let doc = Html::parse_document(html);
    for channel in doc.select(&s(".section.group .colm.span_1_of_3")) {
        let icon_url = channel
            .select(&s("p img"))
            .next()
            .and_then(|icon| icon.value().attr("src"))
            .map(|u| normalize_url(u, DESI_TV))
            .transpose()?;
        let title = channel
            .select(&s("strong"))
            .next()
            .map(|t| t.inner_html())
            .unwrap_or_default();
        let mut tv_channel = TvChannel {
            title: title.clone(),
            icon: icon_url.map(|t| t.into_owned()),
            soaps: Vec::new(),
            completed_soaps: Vec::new(),
        };
        for soap in channel.select(&s("ul li.cat-item a")) {
            let (soap_title, soap_url) = (
                soap.inner_html(),
                soap.value().attr("href").unwrap_or("").to_owned(),
            );
            if soap_url.is_empty() {
                continue;
            }
            let soap_url = normalize_url(&soap_url, DESI_TV)?;
            if soap_title.trim().ends_with("Completed Shows") {
                completed_shows.insert(title, soap_url.into_owned());
                break;
            } else {
                tv_channel.soaps.push(TvSoap {
                    title: soap_title,
                    url: soap_url.into_owned(),
                });
            }
        }
        tv_channels.push(tv_channel);
    }
    Ok((tv_channels, completed_shows))
}

async fn download_icon(href: Option<String>) -> Option<String> {
    if let Some(href) = href {
        let ext = if let Some(idx) = href.rfind('.') {
            &href[idx + 1..]
        } else {
            "jpeg"
        };
        let bytes = download_bytes(&href).await?;
        let res = format!("data:image/{};base64,{}", ext, base64::encode(bytes));
        Some(res)
    } else {
        None
    }
}

async fn download_bytes(href: &str) -> Option<Bytes> {
    Some(
        http_client()
            .get(href)
            .header(header::REFERER, DESI_TV)
            .send()
            .await
            .ok()?
            .bytes()
            .await
            .ok()?,
    )
}

async fn download_completed_shows(url: &str) -> anyhow::Result<Vec<TvSoap>> {
    fn parse(html: &str, host_url: &str) -> Vec<TvSoap> {
        let doc = Html::parse_document(html);
        doc.select(&s(".entry_content ul li ul.children li.cat-item a"))
            .map(|a| (a.inner_html(), a.value().attr("href")))
            .filter_map(|(title, url)| url.map(|url| (title, url)))
            .filter_map(|(title, url)| {
                normalize_url(url, host_url)
                    .map(|url| (title, url.into_owned()))
                    .ok()
            })
            .map(|(title, url)| TvSoap { title, url })
            .collect()
    }
    let html = http_client()
        .get(url)
        .header(header::REFERER, DESI_TV)
        .send()
        .await?
        .text()
        .await?;
    Ok(parse(&html, url))
}

pub async fn get_soap(tv_channel: &str, soap: &str) -> Option<TvSoap> {
    if TV_CHANNEL_STATE.get().is_none() {
        channel_home().await.ok()?;
    }
    let state = TV_CHANNEL_STATE.get()?.tv_channels.read().await;
    let tvc = state.iter().find(|tvc| tvc.title == tv_channel)?;
    tvc.soaps
        .iter()
        .chain(&tvc.completed_soaps)
        .find(|s| s.title == soap)
        .cloned()
}
