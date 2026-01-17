//! RAII guard that executes a closure when dropped.
//!
//! [`DropGuard`] ensures cleanup code runs when a scope exits, regardless of
//! whether it exits normally or due to an early return/panic. This pattern is
//! useful for resource cleanup, state restoration, or notification on scope exit.
//!
//! # Example
//!
//! ```
//! use databend_base::drop_guard::DropGuard;
//!
//! let guard = DropGuard::new(|| println!("cleanup"));
//! // ... do work ...
//! drop(guard); // prints "cleanup"
//! ```
//!
//! # Cancellation
//!
//! Use [`DropGuard::cancel()`] to prevent the callback from running:
//!
//! ```
//! use databend_base::drop_guard::DropGuard;
//!
//! let mut guard = DropGuard::new(|| panic!("should not run"));
//! guard.cancel();
//! drop(guard); // nothing happens
//! ```

use std::fmt;

/// A guard that executes a closure when dropped.
///
/// The closure is guaranteed to run exactly once when the guard is dropped,
/// unless explicitly cancelled via [`cancel()`](Self::cancel).
pub struct DropGuard {
    f: Option<Box<dyn FnOnce() + Send + 'static>>,
}

impl fmt::Debug for DropGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DropGuard")
            .field("active", &self.f.is_some())
            .finish()
    }
}

impl DropGuard {
    /// Creates a new guard that will execute the given closure when dropped.
    pub fn new(f: impl FnOnce() + Send + 'static) -> Self {
        DropGuard {
            f: Some(Box::new(f)),
        }
    }

    /// Cancels the guard, preventing the closure from running on drop.
    ///
    /// This is useful when you want to conditionally skip cleanup,
    /// for example, after a successful operation completes.
    pub fn cancel(&mut self) {
        self.f = None;
    }

    /// Returns `true` if the guard is still active (will run on drop).
    pub fn is_active(&self) -> bool {
        self.f.is_some()
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_executes_on_drop() {
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        {
            let _guard = DropGuard::new(move || {
                called_clone.store(true, Ordering::SeqCst);
            });
            assert!(!called.load(Ordering::SeqCst));
        }
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_cancel_prevents_execution() {
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        {
            let mut guard = DropGuard::new(move || {
                called_clone.store(true, Ordering::SeqCst);
            });
            assert!(guard.is_active());
            guard.cancel();
            assert!(!guard.is_active());
        }
        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_executes_exactly_once() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();
        {
            let _guard = DropGuard::new(move || {
                count_clone.fetch_add(1, Ordering::SeqCst);
            });
        }
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_debug_format() {
        let guard = DropGuard::new(|| {});
        assert!(format!("{:?}", guard).contains("active: true"));

        let mut guard2 = DropGuard::new(|| {});
        guard2.cancel();
        assert!(format!("{:?}", guard2).contains("active: false"));
    }
}
