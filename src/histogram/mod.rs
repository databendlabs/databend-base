#[allow(clippy::module_inception)]
mod histogram;
mod percentile_stats;
mod slot;

pub use histogram::Histogram;
// pub use slot::Slot;
pub use percentile_stats::PercentileStats;
