use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

/// Global sequential ID generator.
///
/// Returns monotonically increasing `usize` values, starting from 0.
/// Thread-safe via atomic operations.
///
/// # Example
/// ```
/// use databend_base::uniq_id::GlobalSeq;
///
/// let a = GlobalSeq::next();
/// let b = GlobalSeq::next();
/// assert!(b > a);
/// ```
pub struct GlobalSeq;

impl GlobalSeq {
    pub fn next() -> usize {
        static GLOBAL_SEQ: AtomicUsize = AtomicUsize::new(0);

        GLOBAL_SEQ.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_increments() {
        let a = GlobalSeq::next();
        let b = GlobalSeq::next();
        assert_eq!(b, a + 1);
    }
}
