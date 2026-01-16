use std::error::Error;
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use futures::Future;
use futures::FutureExt;
use futures::future::BoxFuture;
use log::error;
use log::info;
use tokio::sync::broadcast;

use super::graceful::Graceful;
use crate::unwind::drop_guard;

/// Error returned when shutdown operations fail.
#[derive(Debug, Clone)]
pub enum ShutdownError {
    AlreadyShuttingDown,
}

impl fmt::Display for ShutdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShutdownError::AlreadyShuttingDown => {
                write!(f, "ShutdownGroup is already shutting down")
            }
        }
    }
}

impl std::error::Error for ShutdownError {}

/// Manages graceful shutdown for a group of services.
///
/// Implements two-phase shutdown:
/// - First Ctrl-C triggers graceful shutdown on all services
/// - Second Ctrl-C sends force signal to services
///
/// On drop, triggers force shutdown on all services.
pub struct ShutdownGroup<E: Error + Send + 'static> {
    shutting_down: AtomicBool,
    services: Vec<Box<dyn Graceful<Error = E> + Send>>,
}

impl<E: Error + Send + 'static> ShutdownGroup<E> {
    pub fn new() -> Self {
        ShutdownGroup {
            shutting_down: AtomicBool::new(false),
            services: vec![],
        }
    }

    /// Shutdown all services with optional force signal.
    ///
    /// The `force` future is shared among all services - when it completes,
    /// all services receive the force signal simultaneously.
    #[must_use = "the returned future must be awaited to perform shutdown"]
    pub fn shutdown_all(
        &mut self,
        force: Option<BoxFuture<'static, ()>>,
    ) -> Result<impl Future<Output = ()> + Send + '_, ShutdownError> {
        if self
            .shutting_down
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return Err(ShutdownError::AlreadyShuttingDown);
        }

        let shared = force.map(|f| f.shared());

        let handles: Vec<_> = self
            .services
            .iter_mut()
            .map(|s| s.shutdown(shared.clone().map(|f| f.boxed())))
            .collect();

        let join_all = futures::future::join_all(handles);
        Ok(async move {
            let _ = join_all.await;
        })
    }

    /// Wait for termination signal, then perform two-phase shutdown.
    ///
    /// - First signal: graceful shutdown
    /// - Second signal: force shutdown (passed to services)
    pub fn wait_to_terminate(
        mut self,
        signal: broadcast::Sender<()>,
    ) -> impl Future<Output = ()> + 'static {
        let mut rx = signal.subscribe();

        async move {
            let _ = rx.recv().await;

            info!("Received termination signal.");
            info!("Press Ctrl + C again to force shutdown.");

            let mut force_rx = signal.subscribe();
            let force_fut = async move {
                let _ = force_rx.recv().await;
            }
            .boxed();

            match self.shutdown_all(Some(force_fut)) {
                Ok(f) => f.await,
                Err(e) => info!("Shutdown already in progress: {}", e),
            }
        }
    }

    /// Install Ctrl-C handler that sends signals on the returned channel.
    pub fn install_termination_handle() -> broadcast::Sender<()> {
        let (tx, _rx) = broadcast::channel(16);

        let t = tx.clone();
        ctrlc::set_handler(move || {
            if let Err(error) = t.send(()) {
                error!("Could not send signal on channel {}", error);
                std::process::exit(1);
            }
        })
        .expect("Error setting Ctrl-C handler");

        tx
    }

    pub fn push(&mut self, s: Box<dyn Graceful<Error = E> + Send>) {
        self.services.push(s);
    }
}

impl<E: Error + Send + 'static> Default for ShutdownGroup<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Error + Send + 'static> Drop for ShutdownGroup<E> {
    fn drop(&mut self) {
        drop_guard(move || {
            // Create an immediately-ready future for force shutdown
            let force_fut = async {}.boxed();

            let fut = self.shutdown_all(Some(force_fut));

            if let Ok(fut) = fut {
                futures::executor::block_on(fut);
            }
        })
    }
}
