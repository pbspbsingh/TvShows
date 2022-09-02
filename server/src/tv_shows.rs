use std::collections::HashMap;
use std::time::Instant;

use anyhow::anyhow;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::Json;
use futures::{stream, StreamExt};
use once_cell::sync::OnceCell;
use reqwest::header;
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tracing::*;

use crate::error::HttpError;
use crate::http_util::{find_host, http_client, normalize_url, s, PARALLELISM};
use crate::models::{Episode, TvShow, TvShowEpisodes, VideoProvider};
use crate::tv_channels::get_tv_show;
use crate::utils::fix_title;

static SENDER: OnceCell<UnboundedSender<(TvShow, Sender<TvShowEpisodes>)>> = OnceCell::new();

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

pub async fn init_tv_shows() {
    state::init().await;

    let (sender, mut receiver) = unbounded_channel();
    SENDER.set(sender).unwrap();
    process(&mut receiver).await;
}

async fn process(receiver: &mut UnboundedReceiver<(TvShow, Sender<TvShowEpisodes>)>) {
    let mut stack = Vec::new();
    while let Some(req) = receiver.recv().await {
        stack.push(req);
        while let Ok(req) = receiver.try_recv() {
            stack.push(req);
        }
        while let Some((soap, sender)) = stack.pop() {
            info!("Processing {soap:?}");
            let key = format!("{}:{}", soap.title, soap.url);
            let tv_shows = state::STATE.get().unwrap().get_tv_show(&key).await;
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
            let mut tv_show_episodes = state::STATE
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
                state::STATE
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
    if state::STATE.get().is_none() || SENDER.get().is_none() {
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
    let tv_show = state::STATE.get().unwrap().get_tv_show(&key).await;
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
    tv_show_url: &str,
) -> anyhow::Result<(Vec<(String, Vec<Episode>)>, usize, usize)> {
    fn find_episode_links(html: &str, host: &str) -> (Vec<String>, usize, usize) {
        let doc = Html::parse_document(html);
        let links = doc
            .select(&s(
                "#content_box .latestPost .latestPost-content h2.title a",
            ))
            .filter_map(|e| e.value().attr("href"))
            .filter_map(|href| normalize_url(href, host).ok())
            .map(|href| href.into_owned())
            .collect::<Vec<_>>();
        let current_page = doc
            .select(&s(".nav-links .page-numbers.current"))
            .next()
            .map(|li| li.inner_html())
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1);
        let last_page = doc
            .select(&s(".nav-links .page-numbers:not(.next)"))
            .last()
            .map(|li| li.inner_html())
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(current_page);
        (links, current_page, last_page)
    }

    let response = http_client()
        .get(tv_show_url)
        .header(header::REFERER, find_host(tv_show_url)?)
        .send()
        .await?
        .text()
        .await?;
    let (links, cur_page, last_page) = find_episode_links(&response, tv_show_url);
    info!("Searching for TvShow parts in {links:#?}");
    let episodes = stream::iter(links)
        .map(|link| async move {
            match load_episodes_video_links(&link, tv_show_url).await {
                Ok(res) => Some(res),
                Err(e) => {
                    warn!("Failed to load episodes from {link}: {e}");
                    None
                }
            }
        })
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
    eps_url: &str,
    referer: &str,
) -> anyhow::Result<(String, Vec<Episode>)> {
    fn find_parts(div: ElementRef) -> Option<Episode> {
        let provider = div.select(&s("span.single-heading")).next()?.inner_html();
        let provider = VideoProvider::find(&provider)?;
        let p = ElementRef::wrap(div.next_sibling()?)?;
        let links = p
            .select(&s("a"))
            .map(|a| (a.inner_html(), a.value().attr("href")))
            .filter_map(|(title, opt_link)| opt_link.map(|link| (title, link.to_owned())))
            .collect::<Vec<_>>();
        Some(Episode { provider, links })
    }

    fn find_episode_video_links(html: &str) -> (String, Vec<Episode>) {
        let doc = Html::parse_document(html);
        let title = doc
            .select(&s(".post-single-content header h1.title"))
            .next()
            .map(|t| t.inner_html())
            .unwrap_or_else(|| String::from("NA"));
        let title = fix_title(title);
        let mut parts = doc
            .select(&s(".thecontent div.buttons.btn_green"))
            .filter_map(find_parts)
            .collect::<Vec<_>>();
        parts.sort_by_key(|e| e.provider.priority());
        (title, parts)
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
    let state = state::STATE.get()?;
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

impl VideoProvider {
    pub fn find(text: &str) -> Option<VideoProvider> {
        let text = text.to_uppercase();
        if text.contains("TVLOGY") {
            Some(VideoProvider::TVLogy)
        } else if text.contains("FLASH") {
            Some(VideoProvider::FlashPlayer)
        } else if text.contains("DAILYMOTION") {
            Some(VideoProvider::DailyMotion)
        } else if text.contains("NETFLIX") {
            Some(VideoProvider::NetflixPlayer)
        } else if text.contains("SPEED") {
            Some(VideoProvider::Speed)
        } else if text.contains("VKPRIME") {
            Some(VideoProvider::Vkprime)
        } else {
            None
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            VideoProvider::TVLogy => 1,
            VideoProvider::DailyMotion => 2,
            VideoProvider::NetflixPlayer => 3,
            VideoProvider::FlashPlayer => 4,
            VideoProvider::Speed => 5,
            VideoProvider::Vkprime => 6,
        }
    }
}

mod state {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::process;
    use std::time::SystemTime;

    use once_cell::sync::OnceCell;
    use serde::{Deserialize, Serialize};
    use tokio::fs;
    use tokio::sync::RwLock;
    use tracing::*;

    use crate::models::TvShowEpisodes;
    use crate::utils::{cache_folder, expiry_time, TV_SHOWS_FILE};

    pub(super) static STATE: OnceCell<TvShowsStateWrapper> = OnceCell::new();

    pub(super) struct TvShowsStateWrapper(RwLock<TvShowsState>);

    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct TvShowsState {
        map: HashMap<String, TvShowEpisodes>,
        expires_at: SystemTime,
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
                let state = serde_json::to_string_pretty(state)?;
                fs::write(path, state).await?;
                Ok(())
            }

            debug!("Saving state to file system");
            let path = PathBuf::from(cache_folder()).join(TV_SHOWS_FILE);
            _save_state(path, &*self.0.read().await)
                .await
                .map_err(|e| {
                    error!("Failed to save state to file: {e:?}");
                    process::exit(-1)
                })
                .ok();
        }
    }

    pub async fn init() {
        let path = PathBuf::from(cache_folder());
        if !path.exists() {
            fs::create_dir_all(&path).await.ok();
        }
        let path = path.join(TV_SHOWS_FILE);
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
    }
}
