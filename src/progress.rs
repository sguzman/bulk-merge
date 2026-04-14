use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub log_interval: Duration,
}

#[derive(Debug)]
pub struct ProgressTicker {
    config: ProgressConfig,
    next_log_at: Instant,
}

impl ProgressTicker {
    pub fn new(config: ProgressConfig) -> Self {
        let now = Instant::now();
        let log_interval = config.log_interval;
        Self {
            config,
            next_log_at: now + log_interval,
        }
    }

    pub fn maybe_log(
        &mut self,
        op: &'static str,
        done_units: u64,
        total_units: Option<u64>,
        extra: impl FnOnce(),
    ) {
        let now = Instant::now();
        if now < self.next_log_at {
            return;
        }
        self.next_log_at = now + self.config.log_interval;

        match total_units {
            Some(total) if total > 0 => {
                let pct = (done_units as f64) / (total as f64) * 100.0;
                info!(op, done_units, total_units = total, percent = pct, "progress");
            }
            _ => {
                info!(op, done_units, "progress");
            }
        }
        extra();
    }
}
