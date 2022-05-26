use std::env;

use axum::routing::{any, get};
use axum::{Router, Server};
use mimalloc::MiMalloc;
use structopt::StructOpt;
use tower_http::trace::TraceLayer;
use tracing::*;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::EnvFilter;

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

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const RUST_LOG: &str = "RUST_LOG";

fn main() {
    if env::var_os(RUST_LOG).is_none() {
        env::set_var(
            RUST_LOG,
            "warn,tv_shows_server=debug,tower_http=info,hyper=info",
        );
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(OffsetTime::local_rfc_3339().unwrap())
        .init();

    println!(
        "Version: {}, Log level: {:?}",
        env!("CARGO_PKG_VERSION"),
        env::var_os(RUST_LOG).unwrap()
    );

    _main().ok();
}

#[tokio::main]
async fn _main() -> anyhow::Result<()> {
    let opts = Opts::from_args();
    let address = ([0, 0, 0, 0], opts.port).into();
    info!("Listing for http requests at '{address}'");

    tokio::spawn(tv_shows::init_tv_shows());
    tokio::spawn(cleanup::start_cleanup());
    tokio::spawn(media::download_speed::init());

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
        .fallback(get(file::static_assets))
        .layer(TraceLayer::new_for_http());

    Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "tv_shows_server", about = "Usage of TV show server")]
struct Opts {
    #[structopt(short = "p", long = "port", default_value = "3000")]
    port: u16,
}
