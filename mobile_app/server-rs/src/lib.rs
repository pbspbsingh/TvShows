use android_logger::Config;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use log::{warn, Level};
use tv_shows_server::start_server;

#[no_mangle]
pub extern "system" fn Java_com_pbs_tvshows_server_TvShowsServer_startServer(
    env: JNIEnv,
    _class: JClass,
    cache_folder: jstring,
    async_thread: jint,
    io_thread: jint,
    port: jint,
) -> jstring {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Info)
            .with_tag("TvShowsServer"),
    );

    let cache_folder: String = env
        .get_string(JString::from(cache_folder))
        .expect("Failed to cast java string to rust string")
        .into();
    let async_thread = async_thread as usize;
    let io_thread = io_thread as usize;
    let port = port as u16;
    warn!(
        "Starting server on port: {}, with async threads: {}, blocking threads: {}, cache: {}",
        port, async_thread, io_thread, cache_folder,
    );

    let message = match start_server(&cache_folder, async_thread, io_thread, port) {
        Err(e) => format!("ERROR: {e:?}"),
        Ok(_) => "SUCCESS".into(),
    };
    env.new_string(message)
        .expect("Failed to create java string")
        .into_inner()
}
