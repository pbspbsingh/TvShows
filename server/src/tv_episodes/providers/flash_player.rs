use anyhow::anyhow;
use reqwest::header;
use serde::Deserialize;
use tokio::time::Instant;
use tracing::*;

use crate::http_util::{find_host, http_client};
use crate::tv_episodes::find_iframe;

pub async fn find_m3u8(html: &str, referer: &str) -> anyhow::Result<(String, String)> {
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
    info!(
        "Time taken to resolve FlashPlayer: {}",
        start.elapsed().as_millis()
    );
    Ok((vid_src.file, iframe_src))
}

pub fn find_source(text: &str) -> Option<&str> {
    text.find("sources:").map(|idx| {
        let text = &text[idx..];
        let mut start = 0;
        let mut end = 0;
        for (idx, ch) in text.char_indices() {
            if ch == '{' {
                start = idx;
                break;
            }
        }
        let text = &text[start..];
        let mut stack = 0;
        for (idx, ch) in text.char_indices() {
            end = idx;
            if ch == '{' {
                stack += 1;
            } else if ch == '}' {
                stack -= 1;
            }
            if stack == 0 {
                break;
            }
        }
        &text[..=end]
    })
}

#[derive(Deserialize, Debug)]
struct Source {
    file: String,
}

#[cfg(test)]
mod test {
    use crate::http_util::http_client;

    #[tokio::test]
    async fn test_video_url() -> anyhow::Result<()> {
        let response = http_client()
            .get("https://jumbo.tvlogy.to/tsfiles/DCABFBBF/480K/2022/FIDCBBDA/03/IAEFACFD/11/AGEBCBFF/99289-050.juicycodes")
            .send().await;
        println!("Status: {}", response.is_err());
        Ok(())
    }
}
