use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use std::time::Instant;

use log::Level;
use log::Record;
use pin_project_lite::pin_project;

pin_project! {
    /// A [`Future`] that tracks the time spent on a future.
    /// When the future is ready, the callback will be called with the total time and busy time.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct ElapsedFuture<'a, Fu, F>
    where
        F: FnOnce(&Fu::Output, Duration, Duration),
        F: 'a,
        Fu: Future,
    {
        #[pin]
        inner: Fu,

        busy: Duration,
        // Start time, initialized on first poll.
        start: Option<Instant>,
        // Inspector, consumed when the future completes.
        inspector: Option<F>,
        _p: PhantomData<&'a ()>,
    }
}

impl<'a, Fu, F> ElapsedFuture<'a, Fu, F>
where
    F: FnOnce(&Fu::Output, Duration, Duration),
    F: 'a,
    Fu: Future,
{
    pub fn new(inner: Fu, inspector: F) -> Self {
        Self {
            inner,
            busy: Duration::default(),
            start: None,
            inspector: Some(inspector),
            _p: PhantomData,
        }
    }
}

impl<'a, Fu, F> Future for ElapsedFuture<'a, Fu, F>
where
    F: FnOnce(&Fu::Output, Duration, Duration),
    F: 'a,
    Fu: Future,
{
    type Output = Fu::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // Initialize start time on first poll, only if inspector is present.
        if this.start.is_none() && this.inspector.is_some() {
            *this.start = Some(Instant::now());
        }

        let t0 = Instant::now();
        let res = this.inner.poll(cx);
        *this.busy += t0.elapsed();

        match &res {
            Poll::Ready(output) => {
                if let Some(inspector) = this.inspector.take() {
                    let total = this.start.map(|s| s.elapsed()).unwrap_or_default();
                    (inspector)(output, total, *this.busy);
                }
            }
            Poll::Pending => {}
        }

        res
    }
}

/// Enable elapsed time inspection for a future with `fu.inspect_elapsed(f)`.
pub trait ElapsedFutureExt
where Self: Future
{
    /// Wrap the future to inspect elapsed time.
    fn inspect_elapsed<'a, F>(self, f: F) -> ElapsedFuture<'a, Self, F>
    where
        F: FnOnce(&Self::Output, Duration, Duration) + 'a,
        Self: Future + Sized;

    /// Wrap the future to inspect elapsed time if it exceeds the threshold.
    fn inspect_elapsed_over<'a, F>(
        self,
        threshold: Duration,
        f: F,
    ) -> ElapsedFuture<'a, Self, impl FnOnce(&Self::Output, Duration, Duration)>
    where
        F: FnOnce(&Self::Output, Duration, Duration) + 'a,
        Self: Future + Sized,
    {
        self.inspect_elapsed::<'a>(move |output, total, busy| {
            if total >= threshold {
                f(output, total, busy)
            }
        })
    }

    /// Log elapsed time(total and busy) in DEBUG level when the future is ready.
    #[track_caller]
    fn log_elapsed_debug<'a>(
        self,
        ctx: impl fmt::Display + 'a,
    ) -> ElapsedFuture<'a, Self, impl FnOnce(&Self::Output, Duration, Duration)>
    where
        Self: Future + Sized,
    {
        let caller = Location::caller();
        let caller_file = caller.file();
        let caller_line = caller.line();

        self.inspect_elapsed::<'a>(move |_output, total, busy| {
            if log::log_enabled!(Level::Debug) {
                let args = format_args!("Elapsed: total: {:?}, busy: {:?}; {}", total, busy, ctx);
                let record = Record::builder()
                    .args(args)
                    .level(Level::Debug)
                    .target(module_path!())
                    .file(Some(caller_file))
                    .line(Some(caller_line))
                    .build();
                log::logger().log(&record);
            }
        })
    }

    /// Log elapsed time(total and busy) in info level when the future is ready.
    #[track_caller]
    fn log_elapsed_info<'a>(
        self,
        ctx: impl fmt::Display + 'a,
    ) -> ElapsedFuture<'a, Self, impl FnOnce(&Self::Output, Duration, Duration)>
    where
        Self: Future + Sized,
    {
        let caller = Location::caller();
        let caller_file = caller.file();
        let caller_line = caller.line();

        self.inspect_elapsed::<'a>(move |_output, total, busy| {
            if log::log_enabled!(Level::Info) {
                let args = format_args!("Elapsed: total: {:?}, busy: {:?}; {}", total, busy, ctx);
                let record = Record::builder()
                    .args(args)
                    .level(Level::Info)
                    .target(module_path!())
                    .file(Some(caller_file))
                    .line(Some(caller_line))
                    .build();
                log::logger().log(&record);
            }
        })
    }
}

impl<T> ElapsedFutureExt for T
where T: Future + Sized
{
    fn inspect_elapsed<'a, F>(self, f: F) -> ElapsedFuture<'a, Self, F>
    where
        F: FnOnce(&Self::Output, Duration, Duration),
        F: 'a,
    {
        ElapsedFuture::new(self, f)
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::Context;
    use std::task::Poll;
    use std::time::Duration;

    use crate::futures::ElapsedFuture;
    use crate::futures::ElapsedFutureExt;

    fn build_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap()
    }

    struct BlockingSleep20ms {}

    impl Future for BlockingSleep20ms {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            std::thread::sleep(Duration::from_millis(20));
            Poll::Ready(())
        }
    }

    #[test]
    fn test_elapsed_future_blocking_operation() -> anyhow::Result<()> {
        let rt = build_runtime();

        // blocking_in_place sleep

        let f = async move {
            tokio::task::block_in_place(|| {
                std::thread::sleep(Duration::from_millis(100));
            })
        };
        let f = ElapsedFuture::new(f, |_output, total, busy| {
            // println!("total: {:?}, busy: {:?}", total, busy);
            assert!(total >= Duration::from_millis(100));
            assert!(total <= Duration::from_millis(200));

            assert!(busy >= Duration::from_millis(100));
            assert!(busy <= Duration::from_millis(200));
        });

        rt.block_on(f);

        // blocking_in_place sleep

        #[allow(clippy::disallowed_methods)]
        let f = async move {
            tokio::task::spawn_blocking(|| {
                std::thread::sleep(Duration::from_millis(100));
            })
            .await
            .ok()
        };
        let f = ElapsedFuture::new(f, |_output, total, busy| {
            // println!("total: {:?}, busy: {:?}", total, busy);
            assert!(total >= Duration::from_millis(100));
            assert!(total <= Duration::from_millis(200));

            assert!(busy <= Duration::from_millis(10));
        });

        rt.block_on(f);
        Ok(())
    }

    #[test]
    fn test_elapsed_future() -> anyhow::Result<()> {
        let rt = build_runtime();

        // Blocking sleep

        let f = BlockingSleep20ms {};
        let f = ElapsedFuture::new(f, |_output, total, busy| {
            // println!("total: {:?}, busy: {:?}", total, busy);
            assert!(total >= Duration::from_millis(20));
            assert!(total <= Duration::from_millis(50));

            assert!(busy >= Duration::from_millis(20));
            assert!(busy <= Duration::from_millis(50));
        });

        rt.block_on(f);

        // Async sleep

        let f = async move { tokio::time::sleep(Duration::from_millis(20)).await };
        let f = ElapsedFuture::new(f, |_output, total, busy| {
            // println!("total: {:?}, busy: {:?}", total, busy);
            assert!(total >= Duration::from_millis(20));
            assert!(total <= Duration::from_millis(50));

            assert!(busy <= Duration::from_millis(10));
        });

        rt.block_on(f);

        Ok(())
    }

    #[test]
    fn test_elapsed_future_ext() -> anyhow::Result<()> {
        let rt = build_runtime();

        // Blocking sleep

        let f = BlockingSleep20ms {}.inspect_elapsed(|_output, total, busy| {
            assert!(total >= Duration::from_millis(20));
            assert!(total <= Duration::from_millis(50));

            assert!(busy >= Duration::from_millis(20));
            assert!(busy <= Duration::from_millis(50));
        });

        rt.block_on(f);

        rt.block_on(BlockingSleep20ms {}.inspect_elapsed_over(
            Duration::from_millis(10),
            |_output, _total, _busy| {
                // OK, triggered
            },
        ));
        rt.block_on(
            BlockingSleep20ms {}
                .inspect_elapsed_over(Duration::from_millis(100), |_output, _total, _busy| {
                    unreachable!("should not be called")
                }),
        );

        Ok(())
    }
}
