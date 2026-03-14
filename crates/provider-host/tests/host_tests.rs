//! Unit tests for ProviderHost functionality.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use astragauge_domain::{SensorDescriptor, SensorId};
use astragauge_provider_host::{HostConfig, Provider, ProviderError, ProviderHost};
use astragauge_providers::MockProvider;
use astragauge_sensor_store::SensorStore;

mod common;
use common::{PanickingProvider, PollPanickingProvider};

#[tokio::test]
async fn register_provider_increases_count() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::new_test()));
  let result = host.register_provider(provider);
  assert!(result.is_ok());

  let started_count = host.start();
  assert_eq!(started_count, 1);
}

#[test]
fn duplicate_registration_fails() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider1: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::new_test()));
  let result1 = host.register_provider(provider1);
  assert!(result1.is_ok());

  let provider2: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::new_test()));
  let result2 = host.register_provider(provider2);

  assert!(result2.is_err());
  match result2.unwrap_err() {
    ProviderError::RegistrationFailed { id, reason } => {
      assert_eq!(id, "mock.provider");
      assert!(reason.contains("already registered"));
    }
    _ => panic!("Expected RegistrationFailed error"),
  }
}

#[tokio::test]
async fn samples_appear_in_store() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, Arc::clone(&store));

  let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::new_test()));

  host.register_provider(provider).unwrap();
  let started = host.start();
  assert_eq!(started, 1);

  tokio::time::sleep(Duration::from_millis(50)).await;

  let shutdown_result = host.shutdown().await;
  assert!(shutdown_result.is_ok());

  let sensor_id = SensorId::new("mock.sensor").expect("valid sensor id");
  let latest = store.get_value(&sensor_id).await;
  assert!(
    latest.is_some(),
    "Expected sample to be in store for sensor mock.sensor"
  );
  let sample = latest.unwrap();
  assert_eq!(sample.value, Some(42.0));
}

#[tokio::test]
async fn shutdown_stops_all_providers() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, Arc::clone(&store));

  let sensor1 = SensorId::new("test.sensor1").expect("valid sensor id");
  let descriptor1 = SensorDescriptor {
    id: sensor1.clone(),
    name: "Test Sensor 1".to_string(),
    category: "test".to_string(),
    unit: "units".to_string(),
    device: None,
    tags: vec![],
  };
  let mut values1 = HashMap::new();
  values1.insert(sensor1, 1.0);

  let provider1: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::with_sensors(
    vec![descriptor1],
    values1,
    Duration::from_millis(10),
  )));

  host.register_provider(provider1).unwrap();
  let started = host.start();
  assert_eq!(started, 1);

  tokio::time::sleep(Duration::from_millis(50)).await;

  let shutdown_result = host.shutdown().await;
  assert!(shutdown_result.is_ok());

  let restarted = host.start();
  assert_eq!(restarted, 1);
}

#[tokio::test]
async fn panicking_provider_contained() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(PanickingProvider::new("panic-test")));
  host.register_provider(provider).unwrap();

  host.start();

  tokio::time::sleep(Duration::from_millis(100)).await;

  let shutdown_result = host.shutdown().await;
  assert!(shutdown_result.is_ok());
}

#[tokio::test]
async fn panicking_poll_provider_contained() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  let provider: Arc<Box<dyn Provider>> =
    Arc::new(Box::new(PollPanickingProvider::new("poll-panic-test")));
  host.register_provider(provider).unwrap();

  host.start();

  tokio::time::sleep(Duration::from_millis(100)).await;

  let shutdown_result = host.shutdown().await;
  assert!(shutdown_result.is_ok());
}
