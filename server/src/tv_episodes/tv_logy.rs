use anyhow::anyhow;
use quick_js::{console, Context};
use reqwest::header;
use scraper::Html;
use tokio::time::Instant;
use tracing::*;

use crate::http_util::{find_host, http_client, normalize_url, s};

pub async fn find_m3u8(html: &str, referer: &str) -> anyhow::Result<(String, String)> {
    let start = Instant::now();
    let iframe_src = find_iframe(html).ok_or_else(|| anyhow!("Failed to find iframe"))?;
    debug!("Got iframe src: {iframe_src}");
    let html = http_client()
        .get(&iframe_src)
        .header(header::REFERER, find_host(referer)?)
        .send()
        .await?
        .text()
        .await?;
    let eval_src = find_eval(&html).ok_or_else(|| anyhow!("Couldn't find eval script"))?;
    let (m3u8_url, server) = eval_script(eval_src)?;
    info!(
        "Time taken to resolve TVLogy: {}",
        start.elapsed().as_millis()
    );
    let m3u8_url = format!("{}?s={}&d=", normalize_url(&m3u8_url, &iframe_src)?, server);
    Ok((m3u8_url, iframe_src))
}

fn eval_script(eval_script: &str) -> anyhow::Result<(String, String)> {
    let context = Context::builder().console(console::LogConsole).build()?;
    context.eval(PRELUDES)?;
    context.eval(eval_script)?;
    let video_url = context.eval_as::<String>("videoUrl")?;
    let server = context.eval_as::<String>("server")?;
    Ok((video_url, server))
}

fn find_iframe(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    doc.select(&s("iframe[allowfullscreen]"))
        .next()
        .and_then(|i| i.value().attr("src"))
        .map(ToOwned::to_owned)
}

fn find_eval(html: &str) -> Option<&str> {
    html.find("eval(").map(|start| {
        let text = &html[start..];
        let mut stack = 0;
        let mut end = start;
        for (idx, ch) in text.char_indices() {
            stack += match ch {
                '(' => 1,
                ')' => -1,
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

const PRELUDES: &str = r"
    let videoUrl = '';
    let server = 12;
    
    const document = {};
    
    const FirePlayer = function(a, b, c) {
        videoUrl = b.videoUrl;
        server = b.videoServer;
    };
    
    const $ = function(arg) {
        this.ready = function(a) {
            if (typeof a == 'function') {
                a();
            } else {
                console.log(a);
            }
        };
    
        if (typeof arg == 'function') {
            arg();
        } else {
            console.log(arg);
        }
        return this;	
    };
";
