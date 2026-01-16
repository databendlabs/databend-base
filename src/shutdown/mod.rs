//! Graceful shutdown management for services.
//!
//! This module provides:
//! - [`Graceful`]: Trait for services that support graceful shutdown
//! - [`ShutdownGroup`]: Manager for coordinated shutdown of multiple services

mod graceful;
mod shutdown_group;
#[cfg(test)]
mod shutdown_test;

pub use graceful::Graceful;
pub use shutdown_group::ShutdownError;
pub use shutdown_group::ShutdownGroup;
