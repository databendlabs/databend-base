#[allow(clippy::module_inception)]
mod histogram;
mod log_scale;
mod log_scale_config;
mod percentile_stats;
mod slot;

pub use histogram::Histogram;
pub use log_scale::LOG_SCALE;
pub use log_scale::LogScale;
pub use log_scale::LogScale3;
pub use log_scale_config::DefaultLogScaleConfig;
pub use log_scale_config::LogScaleConfig;
pub use percentile_stats::PercentileStats;
