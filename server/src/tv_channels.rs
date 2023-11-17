use std::time::Instant;

use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt};
use linked_hash_map::LinkedHashMap;
use reqwest::header;
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use tracing::*;

use crate::error::HttpError;
use crate::http_util::{http_client, normalize_url, s, PARALLELISM};
use crate::models::TvShow;
use crate::utils::{encode_uri_component, fix_title};

pub const DESI_TV: &str = "https://www.yodesitv.info";

const NO_OF_CHANNEL_ROWS: usize = 2;

const BANNED_CHANNELS: &[&str] = &["Star Jalsha", "Star Pravah", "Star Vijay", "Bindass TV"];

pub const NO_ICON: &str =
    "https://www.yodesitv.info/wp-content/uploads/2016/11/no-thumbnail-370x208.jpg";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TvShowResponse {
    title: String,
    icon: String,
}

pub async fn channel_home() -> Result<impl IntoResponse, HttpError> {
    async fn _channel_home() -> anyhow::Result<LinkedHashMap<String, Vec<TvShow>>> {
        let state = state::STATE
            .get()
            .ok_or_else(|| anyhow::anyhow!("Wtf, Channel state is not initialized"))?;
        if let Some(tv_channels) = state.get_all_channels().await {
            Ok(tv_channels)
        } else {
            info!("TV channels list have expired, time to refresh it");
            let start = Instant::now();
            let tv_channels = download_tv_channels().await?;
            state.update_state(tv_channels.iter()).await?;
            info!("Time taken to download the tv shows: {:?}", start.elapsed());
            Ok(tv_channels)
        }
    }

    state::init().await;

    let channels = _channel_home().await?;
    let response = channels
        .into_iter()
        .map(|(title, tv_shows)| {
            (
                title,
                tv_shows
                    .into_iter()
                    .map(|TvShow { title, icon, .. }| TvShowResponse { title, icon })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<LinkedHashMap<_, _>>();
    Ok(Json(response))
}

#[instrument]
async fn download_tv_channels() -> anyhow::Result<LinkedHashMap<String, Vec<TvShow>>> {
    info!("Loading TV channels from {DESI_TV}");
    let html = http_client().get(DESI_TV).send().await?.text().await?;
    let mut tv_channels = parse_channels(&html);
    if tv_channels
        .last()
        .map(|(title, _)| title.contains("View All"))
        .unwrap_or(false)
    {
        let (_, link) = tv_channels.pop().unwrap();
        let html = http_client()
            .get(&link)
            .header(header::REFERER, DESI_TV)
            .send()
            .await?
            .text()
            .await?;
        tv_channels.extend(parse_web_series(&html, &link));
    }
    let tv_channels = tv_channels
        .into_iter()
        .filter(|(title, _)| !BANNED_CHANNELS.contains(&(title as &str)))
        .collect::<Vec<_>>();
    info!("Tv channels found: {}", tv_channels.len());

    let mut tv_shows_map = stream::iter(tv_channels)
        .map(|(title, url)| async move {
            let tv_shows = download_tv_shows(&url).await;
            match tv_shows {
                Ok(tv_shows) => Some((title, tv_shows)),
                Err(e) => {
                    warn!("Failed to download tv shows for {title}, {e}");
                    None
                }
            }
        })
        .buffered(PARALLELISM)
        .filter_map(|x| async { x })
        .collect::<LinkedHashMap<_, _>>()
        .await;
    info!(
        "Total Tv Shows: {}",
        tv_shows_map.values().flatten().count()
    );

    for (_, tv_shows) in &mut tv_shows_map {
        for tv_show in tv_shows {
            tv_show.icon = format!("/media?url={}", encode_uri_component(&tv_show.icon));
        }
    }
    Ok(tv_shows_map)
}

fn parse_channels(html: &str) -> Vec<(String, String)> {
    fn find_main_channels(a: ElementRef) -> Option<(String, String)> {
        let mut title = None;
        let link = normalize_url(a.value().attr("href")?, DESI_TV).ok()?;
        let mut prev = a.parent()?.prev_sibling();
        while let Some(p) = prev {
            let p_class = p
                .value()
                .as_element()
                .and_then(|p_ele| p_ele.attr("class"))
                .unwrap_or("");
            if p_class.contains("home-channel-title") {
                let p = ElementRef::wrap(p)?;
                let html = p.select(&s("p")).next()?.inner_html();
                title = Some(fix_title(html));
                break;
            }
            prev = p.prev_sibling();
        }
        Some((title?, link.into_owned()))
    }

    fn find_extra_channels(div: &ElementRef) -> Option<(String, String)> {
        let a = div.select(&s("p.small-title a")).next()?;
        let link = normalize_url(a.value().attr("href")?, DESI_TV).ok()?;
        Some((a.inner_html(), link.into_owned()))
    }

    let mut tv_channels = Vec::new();
    let doc = Html::parse_document(html);
    tv_channels.extend(
        doc.select(&s(
            ".post .single_page .post-content .one_sixth.column-last > a",
        ))
        .filter_map(find_main_channels),
    );
    if tv_channels.len() > NO_OF_CHANNEL_ROWS {
        for _ in 0..NO_OF_CHANNEL_ROWS {
            tv_channels.pop().unwrap();
        }
    }
    tv_channels.extend(
        doc.select(&s(
            ".post .single_page .post-content .one_sixth.column-last",
        ))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .take(NO_OF_CHANNEL_ROWS)
        .rev()
        .flat_map(|mut div| {
            let mut result = Vec::new();
            while let Some(d) = div.prev_sibling() {
                if div
                    .value()
                    .attr("class")
                    .unwrap_or("")
                    .contains("home-channel-title")
                {
                    break;
                }
                if let Some(chn) = find_extra_channels(&div) {
                    result.push(chn);
                }
                div = match ElementRef::wrap(d) {
                    Some(d) => d,
                    None => continue,
                };
            }
            result.reverse();
            result
        }),
    );
    tv_channels
}

fn parse_web_series(html: &str, host: &str) -> Vec<(String, String)> {
    fn parse_anchor(a: ElementRef, host: &str) -> Option<(String, String)> {
        let link = normalize_url(a.value().attr("href")?, host).ok()?;
        Some((a.inner_html(), link.into_owned()))
    }

    let doc = Html::parse_document(html);
    doc.select(&s(".single_page .post-content p[style] a"))
        .filter_map(|a| parse_anchor(a, host))
        .collect()
}

async fn download_tv_shows(url: &str) -> anyhow::Result<Vec<TvShow>> {
    fn parse_tv_show(div: ElementRef, host: &str) -> Option<TvShow> {
        let a = div.select(&s("p.small-title a")).next()?;
        let title = fix_title(a.inner_html());
        let url = normalize_url(a.value().attr("href")?, host)
            .ok()?
            .into_owned();
        let icon = normalize_url(
            div.select(&s("a img"))
                .next()
                .and_then(|img| img.value().attr("src"))
                .unwrap_or(NO_ICON),
            host,
        )
        .ok()?
        .into_owned();
        Some(TvShow { title, url, icon })
    }
    fn parse_tv_shows(html: &str, host: &str) -> Vec<TvShow> {
        let doc = Html::parse_document(html);
        doc.select(&s(".tab_container #tab-0-title-1 .one_fourth"))
            .filter_map(|div| parse_tv_show(div, host))
            .collect()
    }
    info!("Downloading {url}");
    let html = http_client()
        .get(url)
        .header(header::REFERER, DESI_TV)
        .send()
        .await?
        .text()
        .await?;
    Ok(parse_tv_shows(&html, url))
}

pub async fn get_tv_show(tv_channel: &str, tv_show: &str) -> Option<TvShow> {
    state::init().await;
    state::STATE.get()?.get_tv_show(tv_channel, tv_show).await
}

mod state {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use anyhow::anyhow;
    use linked_hash_map::LinkedHashMap;
    use once_cell::sync::OnceCell;
    use serde::*;
    use tokio::fs;
    use tokio::sync::RwLock;
    use tracing::*;

    use crate::models::TvShow;
    use crate::utils::{cache_folder, expiry_time, EXPIRY, TV_CHANNEL_FILE};

    pub static STATE: OnceCell<TvChannelStateWrapper> = OnceCell::new();

    pub struct TvChannelStateWrapper(RwLock<TvChannelState>);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct TvChannelState {
        channels: LinkedHashMap<String, Vec<TvShow>>,
        expires_at: SystemTime,
    }

    impl TvChannelStateWrapper {
        pub async fn get_all_channels(&self) -> Option<LinkedHashMap<String, Vec<TvShow>>> {
            let read = self.0.read().await;
            if read.expires_at >= SystemTime::now() {
                Some(read.channels.clone())
            } else {
                if !read.channels.is_empty() {
                    drop(read);
                    self.0.write().await.channels.clear();
                    self.dump().await.ok();
                }
                None
            }
        }

        pub async fn get_tv_show(&self, tv_channel: &str, tv_show: &str) -> Option<TvShow> {
            self.0
                .read()
                .await
                .channels
                .get(tv_channel)
                .and_then(|v| v.iter().find(|show| show.title == tv_show))
                .cloned()
        }

        pub async fn update_state(
            &self,
            new_channels: impl Iterator<Item = (&String, &Vec<TvShow>)>,
        ) -> anyhow::Result<()> {
            let mut write = self.0.write().await;
            write.channels.clear();
            for (key, value) in new_channels {
                write.channels.insert(key.to_owned(), value.to_owned());
            }
            write.expires_at = expiry_time() + EXPIRY;
            drop(write);
            self.dump().await
        }

        async fn dump(&self) -> anyhow::Result<()> {
            let content = serde_json::to_string_pretty(&*self.0.read().await)?;
            let file = PathBuf::from(cache_folder()).join(TV_CHANNEL_FILE);
            if !file.exists() {
                let parent = file
                    .parent()
                    .ok_or_else(|| anyhow!("ohh man, can't even read cache folder"))?;
                if !parent.exists() {
                    fs::create_dir_all(parent).await?;
                }
            }
            fs::write(file, content).await?;
            Ok(())
        }
    }

    pub async fn init() {
        if STATE.get().is_none() {
            let file = PathBuf::from(cache_folder()).join(TV_CHANNEL_FILE);
            let tv_channels = match fs::read_to_string(&file).await.and_then(|content| {
                serde_json::from_str::<TvChannelState>(&content)
                    .map_err(|_| std::io::ErrorKind::InvalidData.into())
            }) {
                Ok(state) => state,
                Err(e) => {
                    warn!("Couldn't deserialize {file:?}: {e}");
                    if file.exists() {
                        fs::remove_file(&file).await.ok();
                    }
                    TvChannelState {
                        channels: LinkedHashMap::new(),
                        expires_at: SystemTime::now(),
                    }
                }
            };
            STATE
                .set(TvChannelStateWrapper(RwLock::new(tv_channels)))
                .map_err(|_| error!("Duh, couldn't set the state once_cell"))
                .ok();
        }
    }
}
