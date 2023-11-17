use anyhow::anyhow;
use reqwest::header;
use serde::Deserialize;
use tokio::time::Instant;
use tracing::*;

use crate::http_util::{find_host, http_client};

use super::find_iframe;

pub async fn find_m3u8(html: &str, referer: &str) -> anyhow::Result<(String, String)> {
    #[derive(Deserialize, Debug)]
    struct Source {
        file: String,
    }

    let start = Instant::now();
    let iframe_src = find_iframe(html, referer)?;
    debug!("Got iframe src: {iframe_src}");
    let html = http_client()
        .get(&iframe_src)
        .header(header::REFERER, find_host(referer)?)
        .send()
        .await?
        .text()
        .await?;
    let vid_src =
        find_source(&html).ok_or_else(|| anyhow!("Failed to find video source in {iframe_src}"))?;
    let vid_src = serde_json::from_str::<Source>(vid_src)?;
    info!("Time taken to resolve FlashPlayer: {:?}", start.elapsed());
    Ok((vid_src.file, iframe_src))
}

pub fn find_source(text: &str) -> Option<&str> {
    text.find("sources:").map(|idx| {
        let text = &text[idx..];
        let (mut start, mut end) = (0, 0);
        for (idx, ch) in text.char_indices() {
            if ch == '{' {
                start = idx;
                break;
            }
        }
        let text = &text[start..];
        let mut stack = 0;
        for (idx, ch) in text.char_indices() {
            stack += match ch {
                '{' => 1,
                '}' => -1,
                _ => continue,
            };
            if stack == 0 {
                end = idx;
                break;
            }
        }
        &text[..=end]
    })
}
