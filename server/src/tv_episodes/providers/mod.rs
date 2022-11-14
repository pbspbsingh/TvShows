use scraper::Html;

use crate::http_util::{normalize_url, s};

pub mod dailymotion;
pub mod flash_player;
pub mod speed;
pub mod tv_logy;

fn find_iframe(html: &str, base_url: &str) -> anyhow::Result<String> {
    let doc = Html::parse_document(html);
    let url = doc
        .select(&s("iframe[allowfullscreen]"))
        .next()
        .and_then(|i| i.value().attr("src"))
        .ok_or_else(|| anyhow::anyhow!("Failed to find iframe"))?;
    Ok(normalize_url(url, base_url)?.into_owned())
}
