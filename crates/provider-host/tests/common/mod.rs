//! Shared test utilities for provider-host tests.

use std::time::Duration;

use async_trait::async_trait;

use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorSample,
};
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

pub fn make_manifest(id: &str) -> ProviderManifest {
  ProviderManifest {
    id: id.to_string(),
    name: format!("{} provider", id),
    version: "1.0.0".to_string(),
    description: "Test provider".to_string(),
    author: None,
    website: None,
    repository: None,
    license: None,
    tags: None,
    runtime: ">=1.0.0".to_string(),
    capabilities: ProviderCapabilities {
      historical: false,
      high_frequency: false,
      hardware_access: false,
    },
    sensors: SensorCategories { categories: vec![] },
  }
}

pub struct PanickingProvider {
  pub manifest: ProviderManifest,
}

impl PanickingProvider {
  pub fn new(id: &str) -> Self {
    Self {
      manifest: make_manifest(id),
    }
  }
}

#[async_trait]
impl Provider for PanickingProvider {
  fn manifest(&self) -> &ProviderManifest {
    &self.manifest
  }

  fn poll_interval(&self) -> Duration {
    Duration::from_millis(10)
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    panic!("discover() panicked!")
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

pub struct PollPanickingProvider {
  pub manifest: ProviderManifest,
}

impl PollPanickingProvider {
  pub fn new(id: &str) -> Self {
    Self {
      manifest: make_manifest(id),
    }
  }
}

#[async_trait]
impl Provider for PollPanickingProvider {
  fn manifest(&self) -> &ProviderManifest {
    &self.manifest
  }

  fn poll_interval(&self) -> Duration {
    Duration::from_millis(10)
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    Ok(vec![])
  }

  async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
    panic!("poll() panicked!")
  }

  async fn health(&self) -> ProviderHealth {
    ProviderHealth::Ok
  }

  async fn shutdown(&self) -> ProviderResult<()> {
    Ok(())
  }
}
