//! Track the count of active instances of a type `T`.
//!
//! The count is maintained by a [`Counter`] implementation and is incremented/decremented
//! when [`Counted`] is created/dropped.
//!
//! # Example
//!
//! ```ignore
//! struct Connection {}
//! impl Connection {
//!     fn ping(&self) {}
//! }
//!
//! struct MyCounter { identifier: String }
//! impl Counter for MyCounter {
//!     fn incr(&mut self, n: i64) { /* ... */ }
//! }
//!
//! {
//!     let conn = Counted::new(Connection {}, MyCounter { identifier: "db".into() });
//!     // count incremented
//!     conn.ping();
//! } // count decremented on drop
//! ```

use std::ops::Deref;
use std::ops::DerefMut;

/// Defines how to report counter metrics.
pub trait Counter {
    fn incr(&mut self, n: i64);

    /// Create a guard instance that increases the counter when created, and decreases the counter
    /// when dropped.
    fn guard() -> Counted<Self, ()>
    where Self: Default + Sized {
        Counted::new((), Self::default())
    }

    fn counted_guard(self) -> Counted<Self, ()>
    where Self: Sized {
        Counted::new((), self)
    }
}

/// Enable using a closure as a counter: `let _guard = (|n| counter += n).counted_guard();`.
impl<F> Counter for F
where F: FnMut(i64)
{
    fn incr(&mut self, n: i64) {
        self(n)
    }
}

/// Binds a counter to a `T`.
///
/// It counts the number of instances of `T` with the provided [`Counter`].
#[derive(Debug)]
pub struct Counted<C, T>
where C: Counter
{
    counter: C,
    inner: T,
}

impl<C, T> Counted<C, T>
where C: Counter
{
    pub fn new(t: T, counter: C) -> Self {
        let mut s = Self { counter, inner: t };
        s.counter.incr(1);
        s
    }

    pub fn counter(&self) -> &C {
        &self.counter
    }

    pub fn counter_mut(&mut self) -> &mut C {
        &mut self.counter
    }

    /// Extract the wrapped value, consuming the guard.
    ///
    /// The counter is decremented (the guard completes its lifecycle).
    pub fn into_inner(self) -> T {
        use std::mem::ManuallyDrop;
        use std::ptr;

        let mut this = ManuallyDrop::new(self);
        this.counter.incr(-1);
        // SAFETY: `this` is ManuallyDrop so Drop won't run.
        // We read `inner` exactly once, and the struct is consumed.
        unsafe { ptr::read(&this.inner) }
    }

    /// Replace the inner value without affecting the count.
    pub fn replace(&mut self, t: T) -> T {
        std::mem::replace(&mut self.inner, t)
    }
}

/// When being dropped, decreases the count.
impl<C, T> Drop for Counted<C, T>
where C: Counter
{
    fn drop(&mut self) {
        self.counter.incr(-1);
    }
}

/// Let an app use `Counted` the same as using `T`.
impl<C, T> Deref for Counted<C, T>
where C: Counter
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Let an app use `Counted` the same as using `T`.
impl<C, T> DerefMut for Counted<C, T>
where C: Counter
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C: Counter, T> AsRef<T> for Counted<C, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<C: Counter, T> AsMut<T> for Counted<C, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicI64;
    use std::sync::atomic::Ordering;

    use crate::counter::Counted;
    use crate::counter::Counter;

    struct Foo {}
    struct TestCounter {
        n: Arc<AtomicI64>,
    }
    impl Counter for TestCounter {
        fn incr(&mut self, n: i64) {
            self.n.fetch_add(n, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_counted() -> anyhow::Result<()> {
        use Ordering::Relaxed;

        let count = Arc::new(AtomicI64::new(0));
        assert_eq!(0, count.load(Relaxed));

        {
            let _a = Counted::new(Foo {}, TestCounter { n: count.clone() });
            assert_eq!(1, count.load(Relaxed));
            {
                let _b = Counted::new(Foo {}, TestCounter { n: count.clone() });
                assert_eq!(2, count.load(Relaxed));
            }
            assert_eq!(1, count.load(Relaxed));
        }
        assert_eq!(0, count.load(Relaxed));
        Ok(())
    }

    #[test]
    fn test_counted_guard() -> anyhow::Result<()> {
        use Ordering::Relaxed;

        let count = Arc::new(AtomicI64::new(0));
        assert_eq!(0, count.load(Relaxed));

        {
            let _a = (|i: i64| {
                count.fetch_add(i, Relaxed);
            })
            .counted_guard();
            assert_eq!(1, count.load(Relaxed));
            {
                let _b = (|i: i64| {
                    count.fetch_add(i * 2, Relaxed);
                })
                .counted_guard();
                assert_eq!(3, count.load(Relaxed));
            }
            assert_eq!(1, count.load(Relaxed));
        }
        assert_eq!(0, count.load(Relaxed));
        Ok(())
    }

    #[test]
    fn test_into_inner() {
        use Ordering::Relaxed;

        let count = Arc::new(AtomicI64::new(0));
        let guarded = Counted::new(42_i32, TestCounter { n: count.clone() });
        assert_eq!(1, count.load(Relaxed));

        let value = guarded.into_inner();
        assert_eq!(42, value);
        assert_eq!(0, count.load(Relaxed)); // decremented by into_inner
    }

    #[test]
    fn test_counter_mut() {
        use Ordering::Relaxed;

        let count = Arc::new(AtomicI64::new(0));
        let mut guarded = Counted::new((), TestCounter { n: count.clone() });
        assert_eq!(1, count.load(Relaxed));

        // Manually increment via counter_mut
        guarded.counter_mut().incr(5);
        assert_eq!(6, count.load(Relaxed));

        drop(guarded);
        assert_eq!(5, count.load(Relaxed)); // drop decrements by 1
    }

    #[test]
    fn test_replace() {
        use Ordering::Relaxed;

        let count = Arc::new(AtomicI64::new(0));
        let mut guarded = Counted::new(10_i32, TestCounter { n: count.clone() });
        assert_eq!(1, count.load(Relaxed));
        assert_eq!(10, *guarded);

        let old = guarded.replace(20);
        assert_eq!(10, old);
        assert_eq!(20, *guarded);
        assert_eq!(1, count.load(Relaxed)); // count unchanged

        drop(guarded);
        assert_eq!(0, count.load(Relaxed));
    }

    #[test]
    fn test_as_ref_as_mut() {
        let count = Arc::new(AtomicI64::new(0));
        let mut guarded = Counted::new(vec![1, 2, 3], TestCounter { n: count.clone() });

        // AsRef
        let slice: &Vec<i32> = guarded.as_ref();
        assert_eq!(&[1, 2, 3], slice.as_slice());

        // AsMut
        let vec_mut: &mut Vec<i32> = guarded.as_mut();
        vec_mut.push(4);
        assert_eq!(&[1, 2, 3, 4], guarded.as_ref().as_slice());
    }
}
