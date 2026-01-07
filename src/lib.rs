//! Common utilities and data structures for Databend.
//!
//! # Modules
//!
//! - [`histogram`]: A histogram with logarithmic bucketing for tracking u64 value distributions.
//!   Provides O(1) recording and efficient percentile calculation with bounded memory (~2KB).

pub mod histogram;
