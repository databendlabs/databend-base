//! Future utilities and extensions.
//!
//! This module provides utilities for working with async futures:
//! - [`ElapsedFuture`]: A future wrapper that tracks total and busy time.
//! - [`ElapsedFutureExt`]: Extension trait for convenient elapsed time inspection.

mod elapsed;

pub use elapsed::ElapsedFuture;
pub use elapsed::ElapsedFutureExt;
