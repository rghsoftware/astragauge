//! Mock provider for testing purposes.
//!
//! Provides a configurable implementation of the Provider trait
//! that returns predefined sensor descriptors and values.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorId,
  SensorSample,
};
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

#[derive(Clone)]
enum MockPollBehavior {
  Normal,
  Fail(String),
  FailAfter { count: usize, message: String },
}

pub struct MockProvider {
  descriptors: Vec<SensorDescriptor>,
  values: HashMap<SensorId, f64>,
  poll_interval: Duration,
  manifest: ProviderManifest,
  health_state: Arc<Mutex<ProviderHealth>>,
  poll_behavior: Arc<Mutex<MockPollBehavior>>,
  poll_count: Arc<Mutex<usize>>,
}

impl MockProvider {
  pub fn with_sensors(
    descriptors: Vec<SensorDescriptor>,
    values: HashMap<SensorId, f64>,
    poll_interval: Duration,
  ) -> Self {
    Self {
      manifest: create_test_manifest(),
      descriptors,
      values,
      poll_interval,
      health_state: Arc::new(Mutex::new(ProviderHealth::Ok)),
      poll_behavior: Arc::new(Mutex::new(MockPollBehavior::Normal)),
      poll_count: Arc::new(Mutex::new(0)),
    }
  }

  pub fn new_test() -> Self {
    let sensor_id = SensorId::new("mock.sensor").expect("valid sensor id");
    let descriptor = SensorDescriptor {
      id: sensor_id.clone(),
      name: "Mock Sensor".to_string(),
      category: "test".to_string(),
      unit: "units".to_string(),
      device: None,
      tags: vec![],
    };

    let mut values = HashMap::new();
    values.insert(sensor_id, 42.0);

    Self {
      manifest: create_test_manifest(),
      descriptors: vec![descriptor],
      values,
      poll_interval: Duration::from_millis(10),
      health_state: Arc::new(Mutex::new(ProviderHealth::Ok)),
      poll_behavior: Arc::new(Mutex::new(MockPollBehavior::Normal)),
      poll_count: Arc::new(Mutex::new(0)),
    }
  }

  pub fn set_health(&self, health: ProviderHealth) {
    *self.health_state.lock().unwrap() = health;
  }

  pub fn set_poll_failure(&self, message: impl Into<String>) {
    *self.poll_behavior.lock().unwrap() = MockPollBehavior::Fail(message.into());
  }

  pub fn set_poll_failure_after(&self, successful_count: usize, message: impl Into<String>) {
    *self.poll_behavior.lock().unwrap() = MockPollBehavior::FailAfter {
      count: successful_count,
      message: message.into(),
    };
  }

  pub fn poll_count(&self) -> usize {
    *self.poll_count.lock().unwrap()
  }
}

fn create_test_manifest() -> ProviderManifest {
  ProviderManifest {
    id: "mock.provider".to_string(),
    name: "Mock Provider".to_string(),
    version: env!("CARGO_PKG_VERSION").to_string(),
    description: "A mock provider for testing".to_string(),
    author: Some("AstraGauge".to_string()),
    website: None,
    repository: None,
    license: Some("MIT".to_string()),
    tags: Some(vec!["test".to_string(), "mock".to_string()]),
    runtime: ">=0.1.0".to_string(),
    capabilities: ProviderCapabilities {
      historical: false,
      high_frequency: true,
      hardware_access: false,
    },
    sensors: SensorCategories {
      categories: vec!["test".to_string()],
    },
  }
}

#[async_trait]
impl Provider for MockProvider {
  fn manifest(&self) -> &ProviderManifest {
    &self.manifest
  }

  fn poll_interval(&self) -> Duration {
    self.poll_interval
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    Ok(self.descriptors.clone())
  }

  async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
    *self.poll_count.lock().unwrap() += 1;

    let behavior = self.poll_behavior.lock().unwrap().clone();
    match behavior {
      MockPollBehavior::Fail(msg) => {
        return Err(astragauge_provider_host::ProviderError::PollFailed { message: msg });
      }
      MockPollBehavior::FailAfter { count, message } => {
        let current = *self.poll_count.lock().unwrap();
        if current > count {
          return Err(astragauge_provider_host::ProviderError::PollFailed { message });
        }
      }
      MockPollBehavior::Normal => {}
    }

    let timestamp_ms = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0);

    let samples: Vec<SensorSample> = self
      .values
      .iter()
      .map(|(sensor_id, &value)| SensorSample {
        sensor_id: sensor_id.clone(),
        timestamp_ms,
        value: Some(value),
      })
      .collect();

    Ok(samples)
  }

  async fn health(&self) -> ProviderHealth {
    self.health_state.lock().unwrap().clone()
  }

  async fn shutdown(&self) -> ProviderResult<()> {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_test_creates_provider_with_defaults() {
    let provider = MockProvider::new_test();
    assert_eq!(provider.poll_interval(), Duration::from_millis(10));
  }

  #[test]
  fn test_with_sensors_creates_provider_with_custom_config() {
    let sensor_id = SensorId::new("custom.sensor").unwrap();
    let descriptor = SensorDescriptor {
      id: sensor_id.clone(),
      name: "Custom Sensor".to_string(),
      category: "custom".to_string(),
      unit: "units".to_string(),
      device: None,
      tags: vec![],
    };

    let mut values = HashMap::new();
    values.insert(sensor_id.clone(), 100.0);

    let provider = MockProvider::with_sensors(vec![descriptor], values, Duration::from_millis(50));

    assert_eq!(provider.poll_interval(), Duration::from_millis(50));
    assert_eq!(provider.manifest().id, "mock.provider");
  }

  #[tokio::test]
  async fn test_discover_returns_configured_descriptors() {
    let provider = MockProvider::new_test();
    let descriptors = provider.discover().await.unwrap();
    assert_eq!(descriptors.len(), 1);
    assert_eq!(descriptors[0].id.as_str(), "mock.sensor");
  }

  #[tokio::test]
  async fn test_poll_returns_samples_with_timestamps() {
    let provider = MockProvider::new_test();
    let samples = provider.poll().await.unwrap();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].sensor_id.as_str(), "mock.sensor");
    assert!(samples[0].timestamp_ms > 0);
    assert_eq!(samples[0].value, Some(42.0));
  }

  #[tokio::test]
  async fn test_health_returns_ok() {
    let provider = MockProvider::new_test();
    let health = provider.health().await;
    assert_eq!(health, ProviderHealth::Ok);
  }

  #[tokio::test]
  async fn test_shutdown_returns_ok() {
    let provider = MockProvider::new_test();
    let result = provider.shutdown().await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_manifest_returns_test_manifest() {
    let provider = MockProvider::new_test();
    let manifest = provider.manifest();
    assert_eq!(manifest.id, "mock.provider");
    assert_eq!(manifest.name, "Mock Provider");
  }

  #[tokio::test]
  async fn test_configurable_health_degraded() {
    let provider = MockProvider::new_test();
    provider.set_health(ProviderHealth::Degraded {
      message: "sensor unstable".to_string(),
    });
    let health = provider.health().await;
    assert_eq!(
      health,
      ProviderHealth::Degraded {
        message: "sensor unstable".to_string()
      }
    );
  }

  #[tokio::test]
  async fn test_configurable_health_error() {
    let provider = MockProvider::new_test();
    provider.set_health(ProviderHealth::Error {
      message: "device offline".to_string(),
    });
    let health = provider.health().await;
    assert_eq!(
      health,
      ProviderHealth::Error {
        message: "device offline".to_string()
      }
    );
  }

  #[tokio::test]
  async fn test_poll_failure_simulation() {
    let provider = MockProvider::new_test();
    provider.set_poll_failure("simulated failure");
    let result = provider.poll().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
      astragauge_provider_host::ProviderError::PollFailed { message: msg } => {
        assert_eq!(msg, "simulated failure");
      }
      _ => panic!("expected PollFailed error"),
    }
  }

  #[tokio::test]
  async fn test_poll_failure_after_count() {
    let provider = MockProvider::new_test();
    provider.set_poll_failure_after(2, "transient failure");

    assert!(provider.poll().await.is_ok());
    assert!(provider.poll().await.is_ok());

    let result = provider.poll().await;
    assert!(result.is_err());
    assert_eq!(provider.poll_count(), 3);
  }

  #[tokio::test]
  async fn test_poll_count_tracks_calls() {
    let provider = MockProvider::new_test();
    assert_eq!(provider.poll_count(), 0);
    let _ = provider.poll().await;
    assert_eq!(provider.poll_count(), 1);
    let _ = provider.poll().await;
    assert_eq!(provider.poll_count(), 2);
  }
}
