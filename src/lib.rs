//! Common utilities and data structures for Databend.
//!
//! # Modules
//!
//! - [`drop_guard`]: RAII guard that executes a closure when dropped.
//! - [`futures`]: Utilities for working with async futures, including elapsed time tracking.
//! - [`histogram`]: A histogram with logarithmic bucketing for tracking u64 value distributions.
//!   Provides O(1) recording and efficient percentile calculation with bounded memory (~2KB).
//! - [`shutdown`]: Graceful shutdown management for services.
//! - [`uniq_id`]: Unique identifier generators (sequential and random).
//! - [`unwind`]: Panic-safe utilities for handling unwinding scenarios.

pub mod drop_guard;
pub mod futures;
pub mod histogram;
pub mod shutdown;
pub mod uniq_id;
pub mod unwind;
