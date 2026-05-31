use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorSample,
};
use astragauge_provider_host::{HostConfig, Provider, ProviderHealth, ProviderHost};
use astragauge_sensor_store::SensorStore;

mod common;
use common::PanickingProvider;

struct InvalidManifestProvider {
  manifest: ProviderManifest,
}

#[async_trait]
impl Provider for InvalidManifestProvider {
  fn manifest(&self) -> &ProviderManifest {
    &self.manifest
  }

  fn poll_interval(&self) -> Duration {
    Duration::from_millis(10)
  }

  async fn discover(&self) -> astragauge_provider_host::ProviderResult<Vec<SensorDescriptor>> {
    Ok(vec![])
  }

  async fn poll(&self) -> astragauge_provider_host::ProviderResult<Vec<SensorSample>> {
    Ok(vec![])
  }

  async fn health(&self) -> ProviderHealth {
    ProviderHealth::Ok
  }

  async fn shutdown(&self) -> astragauge_provider_host::ProviderResult<()> {
    Ok(())
  }
}

#[tokio::test]
async fn manifest_validation_rejects_empty_id() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(InvalidManifestProvider {
    manifest: ProviderManifest {
      id: "".to_string(),
      name: "Empty ID".to_string(),
      version: "1.0.0".to_string(),
      description: String::new(),
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
    },
  }));

  let result = host.register_provider(provider);
  assert!(result.is_err());
  match result.unwrap_err() {
    astragauge_provider_host::ProviderError::InvalidManifest { reason } => {
      assert!(reason.contains("empty") || reason.contains("non-empty"));
    }
    other => panic!("Expected InvalidManifest, got {:?}", other),
  }
}

#[tokio::test]
async fn unregister_provider_stops_tasks() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> =
    Arc::new(Box::new(astragauge_providers::MockProvider::new_test()));
  host.register_provider(provider).unwrap();
  host.start();

  assert!(host.is_provider_running("mock.provider"));

  let result = host.unregister_provider("mock.provider").await;
  assert!(result.is_ok());
  assert!(!host.is_provider_running("mock.provider"));
  assert!(host.get_provider_health("mock.provider").is_none());
}

#[tokio::test]
async fn unregister_unknown_provider_returns_error() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let result = host.unregister_provider("nonexistent").await;
  assert!(result.is_err());
}

#[tokio::test]
async fn get_provider_health_returns_ok_for_healthy() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> =
    Arc::new(Box::new(astragauge_providers::MockProvider::new_test()));
  host.register_provider(provider).unwrap();

  let health = host.get_provider_health("mock.provider");
  assert!(health.is_some());
  assert!(matches!(health.unwrap(), ProviderHealth::Ok));
}

#[tokio::test]
async fn get_provider_health_returns_error_for_panicking() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> =
    Arc::new(Box::new(PanickingProvider::new("panic-health-test")));
  host.register_provider(provider).unwrap();
  host.start();

  tokio::time::sleep(Duration::from_millis(50)).await;

  let health = host.get_provider_health("panic-health-test");
  assert!(health.is_some());
  assert!(matches!(health.unwrap(), ProviderHealth::Error { .. }));

  host.shutdown().await.unwrap();
}

#[tokio::test]
async fn get_provider_health_returns_none_for_unknown() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = ProviderHost::new(config, store);

  assert!(host.get_provider_health("nonexistent").is_none());
}

#[tokio::test]
async fn is_provider_running_false_before_start() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> =
    Arc::new(Box::new(astragauge_providers::MockProvider::new_test()));
  host.register_provider(provider).unwrap();

  assert!(!host.is_provider_running("mock.provider"));

  host.start();
  assert!(host.is_provider_running("mock.provider"));

  host.shutdown().await.unwrap();
  assert!(!host.is_provider_running("mock.provider"));
}

#[tokio::test]
async fn is_provider_running_false_for_unknown() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = ProviderHost::new(config, store);

  assert!(!host.is_provider_running("nonexistent"));
}
