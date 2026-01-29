//! Common utilities and data structures for Databend.
//!
//! # Modules
//!
//! - [`counter`]: Track the count of active instances with RAII guards.
//! - [`drop_guard`]: RAII guard that executes a closure when dropped.
//! - [`futures`]: Utilities for working with async futures, including elapsed time tracking.
//! - [`histogram`]: A histogram with logarithmic bucketing for tracking u64 value distributions.
//!   Provides O(1) recording and efficient percentile calculation with bounded memory (~2KB).
//! - [`non_empty`]: Non-empty string types that guarantee the contained string is never empty.
//! - [`testutil`]: Utilities for local development and testing, including port allocation.
//! - [`shutdown`]: Graceful shutdown management for services.
//! - [`uniq_id`]: Unique identifier generators (sequential and random).
//! - [`unwind`]: Panic-safe utilities for handling unwinding scenarios.

pub mod counter;
pub mod drop_guard;
pub mod futures;
pub mod histogram;
pub mod non_empty;
pub mod shutdown;
pub mod testutil;
pub mod uniq_id;
pub mod unwind;
