use std::io;

use futures::FutureExt;
use futures::future::BoxFuture;
use log::info;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::time::Duration;

use super::Graceful;
use super::ShutdownGroup;

/// A service that blocks until force shutdown signal.
#[derive(Default)]
struct SlowService {}

#[async_trait::async_trait]
impl Graceful for SlowService {
    type Error = io::Error;

    async fn shutdown(&mut self, force: Option<BoxFuture<'static, ()>>) -> Result<(), Self::Error> {
        info!("--- SlowService shutdown, force: {:?}", force.is_some());

        if let Some(force) = force {
            info!("--- waiting for force");
            force.await;
        }
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_graceful() -> anyhow::Result<()> {
    // - Shutdown blocks until force signal.

    let (stop_tx, mut rx) = broadcast::channel::<()>(1024);
    let (fin_tx, mut fin_rx) = oneshot::channel::<()>();

    let mut svc = SlowService::default();

    let force_fut = async move {
        let _ = rx.recv().await;
    }
    .boxed();

    tokio::spawn(async move {
        let _ = svc.shutdown(Some(force_fut)).await;
        fin_tx.send(()).expect("fail to send fin signal");
    });

    // Shutdown should not return yet.

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(matches!(fin_rx.try_recv(), Err(TryRecvError::Empty)));

    // Send force signal.

    stop_tx.send(()).expect("fail to send force stop");

    assert!(fin_rx.await.is_ok());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_shutdown_group() -> anyhow::Result<()> {
    // - Two services in a group.
    // - First signal triggers graceful shutdown.
    // - Second signal triggers force shutdown.

    let (stop_tx, _) = broadcast::channel::<()>(1024);

    let svc1 = SlowService::default();
    let svc2 = SlowService::default();

    let (fin_tx, mut fin_rx) = oneshot::channel::<()>();

    let mut group = ShutdownGroup::new();
    group.push(Box::new(svc1));
    group.push(Box::new(svc2));

    let fut = group.wait_to_terminate(stop_tx.clone());
    tokio::spawn(async move {
        fut.await;
        fin_tx.send(()).expect("fail to send fin signal");
    });

    info!("--- send graceful stop");
    stop_tx.send(()).expect("fail to set graceful stop");

    // Wait for shutdown() to be called on all services.
    tokio::time::sleep(Duration::from_millis(100)).await;

    info!("--- fin_rx should receive nothing");
    assert!(matches!(fin_rx.try_recv(), Err(TryRecvError::Empty)));

    info!("--- send force stop");
    stop_tx.send(()).expect("fail to set force stop");

    assert!(fin_rx.await.is_ok());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_shutdown_group_drop() -> anyhow::Result<()> {
    // Drop triggers force shutdown - test should not block.

    let svc = SlowService::default();

    let mut group = ShutdownGroup::new();
    group.push(Box::new(svc));

    Ok(())
}
