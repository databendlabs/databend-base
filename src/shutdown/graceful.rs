use futures::future::BoxFuture;

/// A service that supports graceful shutdown.
///
/// Implementors can optionally handle a force shutdown signal
/// if graceful shutdown takes too long.
#[async_trait::async_trait]
pub trait Graceful {
    type Error;

    /// Shutdown the service, waiting until cleanup is complete.
    ///
    /// If graceful shutdown takes too long, the caller may send a force signal
    /// through `force`. Implementations can either shut down immediately
    /// or ignore the signal if force shutdown is not supported.
    async fn shutdown(&mut self, force: Option<BoxFuture<'static, ()>>) -> Result<(), Self::Error>;
}
