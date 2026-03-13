use astragauge_domain::{ProviderManifest, SensorDescriptor, SensorSample};
use async_trait::async_trait;
use std::time::Duration;

use crate::health::ProviderHealth;
use crate::ProviderResult;

/// Core Provider trait - defines the contract for all sensor providers
#[async_trait]
pub trait Provider {
  /// Returns the provider's manifest metadata
  fn manifest(&self) -> &ProviderManifest;

  /// Returns the suggested polling interval for this provider
  fn poll_interval(&self) -> Duration;

  /// Discovers all available sensors from this provider
  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>>;

  /// Polls current values for all registered sensors
  async fn poll(&self) -> ProviderResult<Vec<SensorSample>>;

  /// Returns the current health status of the provider
  async fn health(&self) -> ProviderHealth;

  /// Shuts down the provider and releases resources
  async fn shutdown(&self) -> ProviderResult<()>;
}
