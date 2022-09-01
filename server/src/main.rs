use std::env;

use mimalloc::MiMalloc;
use structopt::StructOpt;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::EnvFilter;

use tv_shows_server::start_server;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const RUST_LOG: &str = "RUST_LOG";

const DEFAULT_LOG_LEVEL: &str = "warn,tv_shows_server=debug,tower_http=info,hyper=info";

fn main() {
    if env::var_os(RUST_LOG).is_none() {
        env::set_var(RUST_LOG, DEFAULT_LOG_LEVEL);
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(OffsetTime::local_rfc_3339().unwrap())
        .init();

    println!(
        "Version: {}, Log level: {:?}",
        env!("CARGO_PKG_VERSION"),
        env::var_os(RUST_LOG).unwrap_or_default()
    );

    let opts = Opts::from_args();
    println!("Program arguments: {opts:?}");

    if let Err(e) = start_server("./cache", opts.async_threads, opts.io_threads, opts.port) {
        eprintln!("Failed to start the server: {e:?}");
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "tv_shows_server", about = "Usage of TV show server")]
struct Opts {
    #[structopt(short = "a", long = "async", default_value = "2")]
    async_threads: usize,
    #[structopt(short = "b", long = "blocking", default_value = "2")]
    io_threads: usize,
    #[structopt(short = "p", long = "port", default_value = "3000")]
    port: u16,
}
