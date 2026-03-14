//! Linux system provider.
//!
//! Provides system metrics from Linux kernel interfaces (/proc, /sys, hwmon).

#[cfg(target_os = "linux")]
use async_trait::async_trait;
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorSample,
};
#[cfg(target_os = "linux")]
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

#[cfg(target_os = "linux")]
pub struct LinuxProvider {
  manifest: ProviderManifest,
  sensors: Vec<SensorDescriptor>,
}

#[cfg(target_os = "linux")]
impl LinuxProvider {
  /// Creates a new Linux provider.
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    Self {
      manifest: linux_manifest(),
      sensors: vec![],
    }
  }
}

#[cfg(target_os = "linux")]
fn linux_manifest() -> ProviderManifest {
  ProviderManifest {
    id: "linux.provider".to_string(),
    name: "Linux System Provider".to_string(),
    version: env!("CARGO_PKG_VERSION").to_string(),
    description: "System metrics from Linux kernel".to_string(),
    author: Some("AstraGauge".to_string()),
    website: None,
    repository: None,
    license: Some("MIT".to_string()),
    tags: Some(vec!["linux".to_string(), "system".to_string()]),
    runtime: ">=0.1.0".to_string(),
    capabilities: ProviderCapabilities {
      historical: false,
      high_frequency: false,
      hardware_access: true,
    },
    sensors: SensorCategories {
      categories: vec![
        "cpu".to_string(),
        "memory".to_string(),
        "temperature".to_string(),
      ],
    },
  }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl Provider for LinuxProvider {
  fn manifest(&self) -> &ProviderManifest {
    &self.manifest
  }

  fn poll_interval(&self) -> Duration {
    Duration::from_millis(1000)
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    Ok(self.sensors.clone())
  }

  async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
    Ok(vec![])
  }

  async fn health(&self) -> ProviderHealth {
    ProviderHealth::Ok
  }

  async fn shutdown(&self) -> ProviderResult<()> {
    Ok(())
  }
}
