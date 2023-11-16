use anyhow::Context;
use axum::routing::{any, get};
use axum::{Router, Server};
use tower_http::trace::TraceLayer;
use tracing::*;

use crate::utils::set_cache_folder;

mod channel_logo;
mod cleanup;
mod error;
mod file;
mod http_util;
mod media;
mod models;
mod tv_channels;
mod tv_episodes;
mod tv_shows;
mod utils;

pub fn start_server(
    cache_dir: &str,
    async_threads: usize,
    io_threads: usize,
    port: u16,
) -> anyhow::Result<()> {
    set_cache_folder(cache_dir)?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(async_threads)
        .max_blocking_threads(io_threads)
        .enable_all()
        .build()?;
    info!(
        "Created tokio runtime with {} async-workers & {} blocking-workers",
        async_threads, io_threads,
    );
    rt.block_on(_start_server(port))?;
    Ok(())
}

async fn _start_server(port: u16) -> anyhow::Result<()> {
    let address = ([0, 0, 0, 0], port).into();
    info!("Listing for http requests at '{address}'");

    tokio::spawn(tv_shows::init_tv_shows());
    tokio::spawn(media::download_speed::init());
    tokio::spawn(cleanup::start_cleanup());

    let app = Router::new()
        .route("/home", get(tv_channels::channel_home))
        .route("/episodes/:tv_channel/:tv_show", get(tv_shows::episodes))
        .route(
            "/episode/:tv_channel/:tv_show/:episode",
            get(tv_episodes::episode_parts),
        )
        .route(
            "/metadata/:folder/:file_name",
            get(tv_episodes::get_metadata),
        )
        .route("/media", any(media::media))
        .route("/logo/:tv_channel", get(channel_logo::logo))
        .fallback(get(file::static_assets))
        .layer(TraceLayer::new_for_http());

    Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .context("Starting Tv show server failed")
}
