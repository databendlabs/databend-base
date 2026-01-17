//! Unique identifier generators.
//!
//! - [`GlobalSeq`]: Sequential IDs (monotonically increasing `usize`)
//! - [`GlobalUniq`]: Random IDs (base62-encoded UUIDv4)

mod seq;
mod uniq;

pub use seq::GlobalSeq;
pub use uniq::GlobalUniq;
