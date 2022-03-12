use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use std::time::{Instant, SystemTime};

use anyhow::anyhow;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt};
use once_cell::sync::OnceCell;
use reqwest::header;
use scraper::Html;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::Sender;
use tokio::sync::{oneshot, RwLock};
use tracing::*;

use crate::error::HttpError;
use crate::http_util::{find_host, http_client, s, PARALLELISM};
use crate::models::{Episode, TvShowEpisodes, TvSoap, VideoProvider};
use crate::tv_channels::get_tv_show;
use crate::utils::{expiry_time, CACHE_FOLDER};

const CACHE_FILE: &str = "tv_shows.json";

static STATE: OnceCell<TvShowsStateWrapper> = OnceCell::new();

static SENDER: OnceCell<UnboundedSender<(TvSoap, oneshot::Sender<TvShowEpisodes>)>> =
    OnceCell::new();

struct TvShowsStateWrapper(RwLock<TvShowsState>);

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TvShowsState {
    map: HashMap<String, TvShowEpisodes>,
    expires_at: SystemTime,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TvShowResponse {
    episodes: Vec<String>,
    has_more: bool,
}

impl TvShowEpisodes {
    fn to_res(&self) -> TvShowResponse {
        let episodes = self.episodes.iter().map(|(eps, _)| eps).cloned().collect();
        let has_more = self.last_page > self.cur_page;
        TvShowResponse { episodes, has_more }
    }
}

impl TvShowsStateWrapper {
    pub async fn get_tv_show(&self, key: &str) -> Option<TvShowEpisodes> {
        let rstate = self.0.read().await;
        if rstate.expires_at < SystemTime::now() {
            drop(rstate);
            warn!("TvShows have already expired, clearing it");

            let mut wstate = self.0.write().await;
            wstate.map.clear();
            wstate.expires_at = expiry_time();
            drop(wstate);

            self.save_state().await;
            None
        } else {
            rstate.map.get(key).cloned()
        }
    }

    pub async fn put_tv_show(&self, key: String, tv_show: TvShowEpisodes) {
        self.0.write().await.map.insert(key, tv_show);
        self.save_state().await;
    }

    async fn save_state(&self) {
        async fn _save_state(path: PathBuf, state: &TvShowsState) -> anyhow::Result<()> {
            let state = serde_json::to_string_pretty(&*state)?;
            fs::write(path, state).await?;
            Ok(())
        }

        debug!("Saving state to file system");
        let path = PathBuf::from(CACHE_FOLDER).join(CACHE_FILE);
        _save_state(path, &*self.0.read().await)
            .await
            .map_err(|e| {
                error!("Failed to save state to file: {e:?}");
                process::exit(-1)
            })
            .ok();
    }
}

pub async fn init_tv_shows() {
    let path = PathBuf::from(CACHE_FOLDER);
    if !path.exists() {
        fs::create_dir_all(&path).await.ok();
    }
    let path = path.join(CACHE_FILE);
    let state = if path.exists() {
        info!("Loading TvShows state from {path:?}");
        match fs::read_to_string(&path).await.and_then(|s| {
            serde_json::from_str(&s).map_err(|_| std::io::ErrorKind::InvalidData.into())
        }) {
            Ok(state) => {
                debug!("Successfully loaded state file");
                state
            }
            Err(e) => {
                warn!("Loading of previously saved state failed: {e:?}");
                fs::remove_file(path)
                    .await
                    .map_err(|e| {
                        error!("Unable to remove the state file: {e:?}");
                        process::exit(-1)
                    })
                    .ok();
                TvShowsState {
                    map: HashMap::new(),
                    expires_at: expiry_time(),
                }
            }
        }
    } else {
        info!("State file doesn't exist");
        TvShowsState {
            map: HashMap::new(),
            expires_at: expiry_time(),
        }
    };
    STATE.set(TvShowsStateWrapper(RwLock::new(state))).ok();

    let (sender, mut receiver) = unbounded_channel();
    SENDER.set(sender).unwrap();
    process(&mut receiver).await;
}

async fn process(receiver: &mut UnboundedReceiver<(TvSoap, Sender<TvShowEpisodes>)>) {
    let mut stack = Vec::new();
    while let Some(req) = receiver.recv().await {
        stack.push(req);
        while let Ok(req) = receiver.try_recv() {
            stack.push(req);
        }
        while let Some((soap, sender)) = stack.pop() {
            info!("Processing {soap:?}");
            let key = format!("{}:{}", soap.title, soap.url);
            let tv_shows = STATE.get().unwrap().get_tv_show(&key).await;
            let soap_url = if let Some(tv_shows) = tv_shows {
                if tv_shows.cur_page == tv_shows.last_page {
                    info!(
                        "All episodes of '{}' has been downloaded already",
                        soap.title
                    );
                    sender.send(tv_shows).ok();
                    continue;
                } else {
                    format!("{}page/{}/", soap.url, tv_shows.cur_page + 1)
                }
            } else {
                soap.url.to_owned()
            };
            let mut tv_show_episodes =
                STATE
                    .get()
                    .unwrap()
                    .get_tv_show(&key)
                    .await
                    .unwrap_or_else(|| TvShowEpisodes {
                        episodes: Vec::new(),
                        cur_page: 1,
                        last_page: 1,
                    });
            info!("Loading episodes from {soap_url}");
            if let Ok((new_episodes, cur_page, last_page)) = load_episodes(&soap_url).await {
                tv_show_episodes.episodes.extend(new_episodes);
                tv_show_episodes.cur_page = cur_page;
                tv_show_episodes.last_page = last_page;
                STATE
                    .get()
                    .unwrap()
                    .put_tv_show(key, tv_show_episodes.clone())
                    .await;
            }
            if sender.send(tv_show_episodes).is_err() {
                warn!("Sending response back failed");
            }

            while let Ok(req) = receiver.try_recv() {
                stack.push(req);
            }
        }
    }
}

pub async fn episodes(
    Path(param): Path<HashMap<String, String>>,
    Query(query_param): Query<HashMap<String, bool>>,
) -> Result<impl IntoResponse, HttpError> {
    if STATE.get().is_none() || SENDER.get().is_none() {
        return Err(anyhow!("State is not initialized yet").into());
    }

    let start = Instant::now();
    let tv_channel = param
        .get("tv_channel")
        .ok_or_else(|| anyhow!("Path didn't contain TvChannel"))?;
    let tv_show = param
        .get("tv_show")
        .ok_or_else(|| anyhow!("Path didn't contain TvShow"))?;
    let &load_more = query_param.get("load_more").unwrap_or(&false);
    info!("Fetching episodes for: {tv_channel} > {tv_show} ({load_more})");
    let soap = get_tv_show(tv_channel, tv_show)
        .await
        .ok_or_else(|| anyhow!("Couldn't find Soap with {tv_channel} & {tv_show}"))?;

    let key = format!("{}:{}", soap.title, soap.url);
    let tv_show = STATE.get().unwrap().get_tv_show(&key).await;
    if let Some(tv_shows) = tv_show {
        info!("Got unexpired TvShows from cache");
        if !load_more {
            return Ok(Json(tv_shows.to_res()));
        }
    }

    let (sender, receiver) = oneshot::channel();
    SENDER
        .get()
        .unwrap()
        .send((soap, sender))
        .map_err(|_| anyhow!("Failed to enqueue the request"))?;
    let response = receiver
        .await
        .map_err(|_| anyhow!("Failed to receive the response from download queue"))?;
    info!(
        "Time taken to serve episodes: {}",
        start.elapsed().as_millis()
    );
    Ok(Json(response.to_res()))
}

async fn load_episodes(
    soap_url: &str,
) -> anyhow::Result<(Vec<(String, Vec<Episode>)>, usize, usize)> {
    fn find_episode_links(html: &str) -> (Vec<String>, usize, usize) {
        let doc = Html::parse_document(html);
        let links = doc
            .select(&s(".post .item_content h4 a"))
            .filter(|e| {
                let link_title = e.inner_html();
                !(link_title.ends_with("Written Update") || link_title.contains("Preview"))
            })
            .filter_map(|e| e.value().attr("href"))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let current_page = doc
            .select(&s("ul.page-numbers li span.page-numbers.current"))
            .next()
            .map(|li| li.inner_html())
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1);
        let mut last_page = doc
            .select(&s(
                "ul.page-numbers li a.page-numbers:not(.prev):not(.next)",
            ))
            .last()
            .map(|li| li.inner_html())
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(current_page);
        if current_page == last_page + 1 {
            last_page = current_page;
        }
        (links, current_page, last_page)
    }

    let response = http_client()
        .get(soap_url)
        .header(header::REFERER, find_host(soap_url)?)
        .send()
        .await?
        .text()
        .await?;
    let (links, cur_page, last_page) = find_episode_links(&response);
    let episodes = stream::iter(links)
        .map(|link| async move { load_episodes_video_links(link, soap_url).await.ok() })
        .buffered(PARALLELISM)
        .collect::<Vec<_>>()
        .await;
    let mut map = HashMap::with_capacity(episodes.len());
    let filtered_episodes = episodes
        .into_iter()
        .flatten()
        .filter_map(|(name, eps)| {
            if eps.is_empty() {
                return None;
            }
            let new_name = if map.contains_key(&name) {
                map.insert(name.clone(), map[&name] + 1);
                format!("{} - {}", name, map[&name])
            } else {
                map.insert(name.clone(), 1);
                name
            };
            Some((new_name, eps))
        })
        .collect::<Vec<_>>();
    Ok((filtered_episodes, cur_page, last_page))
}

async fn load_episodes_video_links(
    eps_url: String,
    referer: &str,
) -> anyhow::Result<(String, Vec<Episode>)> {
    fn update_episodes(
        episodes: &mut Vec<Episode>,
        cur_provider: &mut Option<VideoProvider>,
        videos: &mut Vec<(String, String)>,
    ) {
        if cur_provider.is_some() && !videos.is_empty() {
            episodes.push(Episode {
                provider: cur_provider.unwrap(),
                links: videos.drain(..).collect(),
            })
        }
        *cur_provider = None;
        videos.clear();
    }

    fn find_episode_video_links(html: &str) -> (String, Vec<Episode>) {
        let doc = Html::parse_document(html);
        let mut episodes = Vec::new();
        let mut cur_provider = None;
        let mut videos = Vec::new();
        for p in doc.select(&s(
            ".post .shortcode-content .entry_content p:not(#replace1)",
        )) {
            if let Some(provider) = p.select(&s("b span")).next() {
                update_episodes(&mut episodes, &mut cur_provider, &mut videos);
                if let Some(provider) = VideoProvider::find(&provider.inner_html()) {
                    cur_provider = Some(provider);
                }
            } else if cur_provider.is_some() {
                let links = p.select(&s("a")).collect::<Vec<_>>();
                if links.is_empty() {
                    update_episodes(&mut episodes, &mut cur_provider, &mut videos);
                } else {
                    for link in links {
                        if let Some(url) = link.value().attr("href") {
                            videos.push((link.inner_html(), url.to_owned()));
                        }
                    }
                }
            }
        }
        update_episodes(&mut episodes, &mut cur_provider, &mut videos);
        let title = doc
            .select(&s("h1.name.entry_title span"))
            .next()
            .map(|t| t.inner_html())
            .unwrap_or_else(|| String::from("NA"));
        (title, episodes)
    }
    let response = http_client()
        .get(eps_url)
        .header(header::REFERER, referer)
        .send()
        .await?
        .text()
        .await?;
    Ok(find_episode_video_links(&response))
}

pub async fn get_episode_parts(
    tv_channel: &str,
    tv_show: &str,
    title: &str,
) -> Option<Vec<Episode>> {
    let soap = get_tv_show(tv_channel, tv_show).await?;
    let state = STATE.get()?;
    let episodes = state
        .get_tv_show(&format!("{}:{}", soap.title, soap.url))
        .await?;
    let eps = episodes
        .episodes
        .into_iter()
        .find(|(t, _)| t == title)
        .map(|(_, eps)| eps)?;
    Some(eps)
}
