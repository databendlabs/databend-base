//! Common utilities and data structures for Databend.
//!
//! # Modules
//!
//! - [`futures`]: Utilities for working with async futures, including elapsed time tracking.
//! - [`histogram`]: A histogram with logarithmic bucketing for tracking u64 value distributions.
//!   Provides O(1) recording and efficient percentile calculation with bounded memory (~2KB).
//! - [`shutdown`]: Graceful shutdown management for services.
//! - [`unwind`]: Panic-safe utilities for handling unwinding scenarios.

pub mod futures;
pub mod histogram;
pub mod shutdown;
pub mod unwind;
