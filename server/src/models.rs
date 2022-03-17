use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvShow {
    pub title: String,
    pub url: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvShowEpisodes {
    pub episodes: Vec<(String, Vec<Episode>)>,
    pub cur_page: usize,
    pub last_page: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Episode {
    pub provider: VideoProvider,
    pub links: Vec<(String, String)>,
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
