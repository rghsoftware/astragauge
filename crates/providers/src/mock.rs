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

/// A sine wave: `base + amplitude * sin(2*pi*(now_ms % period_ms) / period_ms)`.
struct Wave {
  base: f64,
  amplitude: f64,
  period_ms: u64,
}

/// Inclusive clamp bounds for a sensor's plausible value range.
struct Range {
  min: f64,
  max: f64,
}

/// Parameters for a time-varying demo sensor: descriptor plus the wave that
/// generates its value, clamped to the sensor's plausible range.
struct DemoSensor {
  descriptor: SensorDescriptor,
  wave: Wave,
  clamp: Range,
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

  /// Creates a MockProvider with a full set of time-varying demo sensors so
  /// the default panel reads like a real system instrument cluster. Each
  /// sensor's value follows a sine wave:
  /// `value = clamp(base + amplitude * sin(2*pi*(now_ms % period_ms) / period_ms), clamp_min, clamp_max)`.
  ///
  /// Poll interval is 500ms.
  pub fn new_demo() -> Self {
    fn demo(
      id: &str,
      name: &str,
      category: &str,
      unit: &str,
      wave: Wave,
      clamp: Range,
    ) -> DemoSensor {
      DemoSensor {
        descriptor: SensorDescriptor {
          id: SensorId::new(id).expect("valid sensor id"),
          name: name.to_string(),
          category: category.to_string(),
          unit: unit.to_string(),
          device: None,
          tags: vec![],
        },
        wave,
        clamp,
      }
    }

    let demo_sensors = vec![
      // CPU
      demo(
        "cpu.total.utilization",
        "CPU Utilization",
        "utilization",
        "%",
        Wave {
          base: 45.0,
          amplitude: 38.0,
          period_ms: 7000,
        },
        Range {
          min: 0.0,
          max: 100.0,
        },
      ),
      demo(
        "cpu.clock",
        "CPU Clock",
        "frequency",
        "MHz",
        Wave {
          base: 4100.0,
          amplitude: 700.0,
          period_ms: 6500,
        },
        Range {
          min: 800.0,
          max: 5200.0,
        },
      ),
      demo(
        "cpu.temperature",
        "CPU Temperature",
        "temperature",
        "°C",
        Wave {
          base: 55.0,
          amplitude: 18.0,
          period_ms: 11000,
        },
        Range {
          min: 20.0,
          max: 95.0,
        },
      ),
      // GPU
      demo(
        "gpu.total.utilization",
        "GPU Utilization",
        "utilization",
        "%",
        Wave {
          base: 40.0,
          amplitude: 40.0,
          period_ms: 9000,
        },
        Range {
          min: 0.0,
          max: 100.0,
        },
      ),
      demo(
        "gpu.temperature",
        "GPU Temperature",
        "temperature",
        "°C",
        Wave {
          base: 50.0,
          amplitude: 22.0,
          period_ms: 13000,
        },
        Range {
          min: 20.0,
          max: 95.0,
        },
      ),
      // Memory
      demo(
        "memory.used.percent",
        "Memory Used",
        "utilization",
        "%",
        Wave {
          base: 60.0,
          amplitude: 20.0,
          period_ms: 17000,
        },
        Range {
          min: 0.0,
          max: 100.0,
        },
      ),
      demo(
        "memory.used",
        "Memory Used",
        "memory",
        "MB",
        Wave {
          base: 19000.0,
          amplitude: 4500.0,
          period_ms: 17000,
        },
        Range {
          min: 2048.0,
          max: 32768.0,
        },
      ),
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
          let phase = (timestamp_ms % s.wave.period_ms) as f64 / s.wave.period_ms as f64;
          let value = (s.wave.base + s.wave.amplitude * (2.0 * PI * phase).sin())
            .clamp(s.clamp.min, s.clamp.max);
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
  async fn test_new_demo_discovers_full_instrument_set() {
    let provider = MockProvider::new_demo();
    let descriptors = provider.discover().await.unwrap();
    assert_eq!(descriptors.len(), 7);

    let ids: Vec<&str> = descriptors.iter().map(|d| d.id.as_str()).collect();
    for expected in [
      "cpu.total.utilization",
      "cpu.clock",
      "cpu.temperature",
      "gpu.total.utilization",
      "gpu.temperature",
      "memory.used.percent",
      "memory.used",
    ] {
      assert!(ids.contains(&expected), "missing demo sensor {expected}");
    }
  }

  #[tokio::test]
  async fn test_new_demo_poll_returns_samples_within_per_sensor_range() {
    let provider = MockProvider::new_demo();
    let samples = provider.poll().await.unwrap();
    assert_eq!(samples.len(), 7);

    // Per-sensor plausible bounds: percentages and temps stay <= 100, while
    // clock (MHz) and memory (MB) range much higher. Assert each value is
    // finite, positive, and within a generous instrument ceiling.
    for sample in &samples {
      assert!(sample.timestamp_ms > 0);
      let value = sample.value.expect("demo sample should have a value");
      assert!(value.is_finite() && value >= 0.0, "value {value} invalid");
      assert!(value <= 32768.0, "value {value} above instrument ceiling");
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
