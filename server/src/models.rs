use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvChannel {
    pub title: String,
    pub icon: Option<String>,
    pub soaps: Vec<TvSoap>,
    pub completed_soaps: Vec<TvSoap>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvSoap {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Episode {
    pub provider: VideoProvider,
    pub links: Vec<(String, String)>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvShowEpisodes {
    pub episodes: Vec<(String, Vec<Episode>)>,
    pub cur_page: usize,
    pub last_page: usize,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum VideoProvider {
    TVLogy,
    FlashPlayer,
    DailyMotion,
    NetflixPlayer,
    Speed,
    Vkprime,
}

impl VideoProvider {
    pub fn find(text: &str) -> Option<VideoProvider> {
        if text.contains("TVLogy") {
            Some(VideoProvider::TVLogy)
        } else if text.contains("Flash Player") {
            Some(VideoProvider::FlashPlayer)
        } else if text.contains("Dailymotion") {
            Some(VideoProvider::DailyMotion)
        } else if text.contains("Netflix Player") {
            Some(VideoProvider::NetflixPlayer)
        } else if text.contains("Speed") {
            Some(VideoProvider::Speed)
        } else if text.contains("Vkprime") {
            Some(VideoProvider::Vkprime)
        } else {
            None
        }
    }
}
