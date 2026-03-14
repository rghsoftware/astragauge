#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HostConfig {
  pub default_poll_interval_ms: u64,
  pub min_poll_interval_ms: u64,
  pub shutdown_timeout_ms: u64,
}

impl HostConfig {
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[must_use]
  pub fn with_default_poll_interval(mut self, interval_ms: u64) -> Self {
    self.default_poll_interval_ms = interval_ms;
    self
  }

  #[must_use]
  pub fn with_shutdown_timeout(mut self, timeout_ms: u64) -> Self {
    self.shutdown_timeout_ms = timeout_ms;
    self
  }
}

impl Default for HostConfig {
  fn default() -> Self {
    Self {
      default_poll_interval_ms: 1000,
      min_poll_interval_ms: 100,
      shutdown_timeout_ms: 10000,
    }
  }
}
