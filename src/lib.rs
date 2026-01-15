//! Common utilities and data structures for Databend.
//!
//! # Modules
//!
//! - [`futures`]: Utilities for working with async futures, including elapsed time tracking.
//! - [`histogram`]: A histogram with logarithmic bucketing for tracking u64 value distributions.
//!   Provides O(1) recording and efficient percentile calculation with bounded memory (~2KB).

pub mod futures;
pub mod histogram;
