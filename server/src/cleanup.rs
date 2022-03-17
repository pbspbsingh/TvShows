use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::{Duration, SystemTime};

use tokio::{fs, time};
use tracing::*;

use crate::utils::{expiry_time, CACHE_FOLDER, TV_CHANNEL_FILE, TV_SHOWS_FILE, WEEK};

const DO_NOT_DELETE_FILES: &[&str] = &[CACHE_FOLDER, TV_CHANNEL_FILE, TV_SHOWS_FILE];

pub async fn start_cleanup() -> ! {
    async fn cleanup() -> anyhow::Result<()> {
        info!("Running cleanup task...");
        let deleted_count = dfs(PathBuf::from(CACHE_FOLDER)).await?;
        if deleted_count > 0 {
            info!("Cleaned {deleted_count} expired files/folders");
        }
        Ok(())
    }

    fn dfs(path: PathBuf) -> Pin<Box<dyn Future<Output = anyhow::Result<u32>> + Send>> {
        Box::pin(async {
            let mut count = 0;
            let metadata = fs::metadata(&path).await?;
            if metadata.is_dir() {
                let mut read_dir = fs::read_dir(&path).await?;
                while let Some(child) = read_dir.next_entry().await? {
                    count += dfs(child.path()).await?;
                }
                let mut read_dir = fs::read_dir(&path).await?;
                if read_dir.next_entry().await?.is_none() {
                    count += delete(path).await?;
                }
            } else if metadata.is_file() && metadata.modified()?.elapsed()? > WEEK {
                count += delete(path).await?;
            }
            Ok(count)
        })
    }

    async fn delete(path: PathBuf) -> anyhow::Result<u32> {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if DO_NOT_DELETE_FILES.contains(&name) {
            return Ok(0);
        }

        if path.is_dir() {
            debug!("Deleting empty dir: {path:?}");
            fs::remove_dir(path).await?;
        } else {
            debug!("Deleting file: {path:?}",);
            fs::remove_file(path).await?;
        }
        Ok(1)
    }

    loop {
        cleanup()
            .await
            .map_err(|e| warn!("Cleanup task failed: {e}"))
            .ok();
        let sleep_dur = expiry_time().duration_since(SystemTime::now()).unwrap();
        debug!("Cleanup task sleeping for {}", fmt(sleep_dur));
        time::sleep(sleep_dur).await;
    }
}

fn fmt(dur: Duration) -> String {
    let mut seconds = dur.as_secs();
    let hours = seconds / (60 * 60);
    seconds -= hours * 60 * 60;
    let minutes = seconds / 60;
    seconds -= minutes * 60;

    let mut result = Vec::new();
    if hours > 0 {
        result.push(hours);
    }
    result.push(minutes);
    result.push(seconds);
    result
        .into_iter()
        .map(|x| x.to_string())
        .map(|x| if x.len() == 1 { format!("0{}", x) } else { x })
        .collect::<Vec<_>>()
        .join(":")
}
