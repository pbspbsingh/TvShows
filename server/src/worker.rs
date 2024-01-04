use futures::future::BoxFuture;
use futures::FutureExt;
use once_cell::sync::OnceCell;
use std::future::Future;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, oneshot};
use tokio::time;
use tracing::{debug, error};

static TASK_SENDER: OnceCell<UnboundedSender<BoxFuture<'static, ()>>> = OnceCell::new();

pub async fn init_worker() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    TASK_SENDER.set(tx).expect("Failed setting the TASK_SENDER");

    let mut task_count = 1;
    while let Some(task) = rx.recv().await {
        debug!("Executing next task: {task_count}");
        task_count += 1;
        if let Err(e) = time::timeout(Duration::from_secs(30), task).await {
            error!("Timeout while executing an async task: {e}");
        }
    }
}

/// Runs the async task in sequence.
pub async fn run<T: Send + 'static>(
    job: impl Future<Output = anyhow::Result<T>> + Send + 'static,
) -> anyhow::Result<T> {
    let (tx, rx) = oneshot::channel::<anyhow::Result<T>>();

    let task = async move {
        tx.send(job.await).ok();
    }
    .boxed();
    let _ = TASK_SENDER
        .get()
        .ok_or_else(|| anyhow::anyhow!("TASK_SENDER is not yet initialized"))
        .and_then(|task_sender| {
            task_sender
                .send(task)
                .map_err(|_| anyhow::anyhow!("Couldn't send the task to TASK_SENDER"))
        })?;
    rx.await?
}
