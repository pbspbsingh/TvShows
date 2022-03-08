use std::env;

use axum::routing::get;
use axum::{Router, Server};
use mimalloc::MiMalloc;
use structopt::StructOpt;
use tower_http::trace::TraceLayer;
use tracing::*;

mod error;
mod http_util;
mod models;
mod tv_channels;
mod tv_shows;
mod utils;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const RUST_LOG: &str = "RUST_LOG";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os(RUST_LOG).is_none() {
        env::set_var(
            RUST_LOG,
            "warn,tv_shows_server=debug,tower_http=info,hyper=info",
        );
    }
    tracing_subscriber::fmt::init();
    println!("Log level: {:?}", env::var_os(RUST_LOG).unwrap());

    let opts = Opts::from_args();
    let address = ([0, 0, 0, 0], opts.port).into();
    info!("Listing for http requests at '{address}'");

    tokio::spawn(tv_shows::init_tv_shows());
    let app = Router::new()
        .route("/home", get(tv_channels::channel_home))
        .route("/episodes/:tv_channel/:soap", get(tv_shows::episodes))
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
