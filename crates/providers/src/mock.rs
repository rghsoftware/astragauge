//! Mock provider for testing purposes.
//!
//! Provides a configurable implementation of the Provider trait
//! that returns predefined sensor descriptors and values.

use async_trait::async_trait;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorId,
  SensorSample,
};
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

/// Parameters for a time-varying demo sensor: descriptor plus the
/// (base, amplitude, period_ms) used to compute a sine-based value.
struct DemoSensor {
  descriptor: SensorDescriptor,
  base: f64,
  amplitude: f64,
  period_ms: u64,
}

/// A mock provider for testing that returns configurable sensor data.
pub struct MockProvider {
  descriptors: Vec<SensorDescriptor>,
  values: HashMap<SensorId, f64>,
  poll_interval: Duration,
  manifest: ProviderManifest,
  /// When non-empty, `poll()` emits time-varying sine values from these
  /// instead of the static `values` map.
  demo_sensors: Vec<DemoSensor>,
}

impl MockProvider {
  /// Creates a new MockProvider with the given configuration.
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
      demo_sensors: Vec::new(),
    }
  }

  /// Creates a MockProvider with sensible defaults for testing.
  ///
  /// Default configuration:
  /// - 1 sensor: `mock.sensor` with value 42.0
  /// - 10ms poll interval
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
      demo_sensors: Vec::new(),
    }
  }

  /// Creates a MockProvider with four time-varying demo sensors so the UI
  /// visibly updates. Each sensor's value follows a sine wave:
  /// `value = clamp(base + amplitude * sin(2*pi*(now_ms % period_ms) / period_ms), 0, 100)`.
  ///
  /// Poll interval is 500ms.
  pub fn new_demo() -> Self {
    let demo_sensors = vec![
      DemoSensor {
        descriptor: SensorDescriptor {
          id: SensorId::new("cpu.total.utilization").expect("valid sensor id"),
          name: "CPU Utilization".to_string(),
          category: "utilization".to_string(),
          unit: "percent".to_string(),
          device: None,
          tags: vec![],
        },
        base: 45.0,
        amplitude: 35.0,
        period_ms: 7000,
      },
      DemoSensor {
        descriptor: SensorDescriptor {
          id: SensorId::new("cpu.temperature").expect("valid sensor id"),
          name: "CPU Temperature".to_string(),
          category: "temperature".to_string(),
          unit: "celsius".to_string(),
          device: None,
          tags: vec![],
        },
        base: 55.0,
        amplitude: 18.0,
        period_ms: 11000,
      },
      DemoSensor {
        descriptor: SensorDescriptor {
          id: SensorId::new("memory.used.percent").expect("valid sensor id"),
          name: "Memory Used".to_string(),
          category: "utilization".to_string(),
          unit: "percent".to_string(),
          device: None,
          tags: vec![],
        },
        base: 60.0,
        amplitude: 20.0,
        period_ms: 17000,
      },
      DemoSensor {
        descriptor: SensorDescriptor {
          id: SensorId::new("gpu.temperature").expect("valid sensor id"),
          name: "GPU Temperature".to_string(),
          category: "temperature".to_string(),
          unit: "celsius".to_string(),
          device: None,
          tags: vec![],
        },
        base: 50.0,
        amplitude: 22.0,
        period_ms: 13000,
      },
    ];

    let descriptors = demo_sensors.iter().map(|s| s.descriptor.clone()).collect();

    Self {
      manifest: create_test_manifest(),
      descriptors,
      values: HashMap::new(),
      poll_interval: Duration::from_millis(500),
      demo_sensors,
    }
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
    let timestamp_ms = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0);

    if !self.demo_sensors.is_empty() {
      let samples: Vec<SensorSample> = self
        .demo_sensors
        .iter()
        .map(|s| {
          let phase = (timestamp_ms % s.period_ms) as f64 / s.period_ms as f64;
          let value = (s.base + s.amplitude * (2.0 * PI * phase).sin()).clamp(0.0, 100.0);
          SensorSample {
            sensor_id: s.descriptor.id.clone(),
            timestamp_ms,
            value: Some(value),
          }
        })
        .collect();
      return Ok(samples);
    }

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
    ProviderHealth::Ok
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

  #[test]
  fn test_new_demo_has_500ms_poll_interval() {
    let provider = MockProvider::new_demo();
    assert_eq!(provider.poll_interval(), Duration::from_millis(500));
  }

  #[tokio::test]
  async fn test_new_demo_discovers_four_sensors() {
    let provider = MockProvider::new_demo();
    let descriptors = provider.discover().await.unwrap();
    assert_eq!(descriptors.len(), 4);

    let ids: Vec<&str> = descriptors.iter().map(|d| d.id.as_str()).collect();
    assert!(ids.contains(&"cpu.total.utilization"));
    assert!(ids.contains(&"cpu.temperature"));
    assert!(ids.contains(&"memory.used.percent"));
    assert!(ids.contains(&"gpu.temperature"));
  }

  #[tokio::test]
  async fn test_new_demo_poll_returns_four_samples_in_range() {
    let provider = MockProvider::new_demo();
    let samples = provider.poll().await.unwrap();
    assert_eq!(samples.len(), 4);

    for sample in &samples {
      assert!(sample.timestamp_ms > 0);
      let value = sample.value.expect("demo sample should have a value");
      assert!(
        (0.0..=100.0).contains(&value),
        "value {} out of range",
        value
      );
    }
  }

  #[tokio::test]
  async fn test_manifest_returns_test_manifest() {
    let provider = MockProvider::new_test();
    let manifest = provider.manifest();
    assert_eq!(manifest.id, "mock.provider");
    assert_eq!(manifest.name, "Mock Provider");
  }
}
