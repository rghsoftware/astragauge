use crate::validation::SensorId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SensorDescriptor {
  pub id: SensorId,
  pub name: String,
  pub category: String,
  pub unit: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub device: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SensorSample {
  pub sensor_id: SensorId,
  pub timestamp_ms: u64,
  /// Sensor value, or None if the sensor is missing/unavailable.
  /// Per sensor-store.md spec: "missing sensor" should return null/undefined.
  pub value: Option<f64>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sensor_descriptor_json() {
    let json = r#"{
            "id": "cpu.temperature",
            "name": "CPU Temperature",
            "category": "temperature",
            "unit": "celsius"
        }"#;
    let descriptor: SensorDescriptor = serde_json::from_str(json).unwrap();
    assert_eq!(descriptor.name, "CPU Temperature");
  }

  #[test]
  fn test_sensor_descriptor_with_optional() {
    let json = r#"{
            "id": "cpu.temperature",
            "name": "CPU Temperature",
            "category": "temperature",
            "unit": "celsius",
            "device": "cpu0",
            "tags": ["thermal"]
        }"#;
    let descriptor: SensorDescriptor = serde_json::from_str(json).unwrap();
    assert_eq!(descriptor.device, Some("cpu0".to_string()));
    assert_eq!(descriptor.tags, vec!["thermal"]);
  }

  #[test]
  fn test_sensor_sample_json() {
    let json = r#"{
            "sensor_id": "cpu.temperature",
            "timestamp_ms": 1712345678,
            "value": 72.3
        }"#;
    let sample: SensorSample = serde_json::from_str(json).unwrap();
    assert_eq!(sample.timestamp_ms, 1712345678);
    assert!((sample.value.unwrap() - 72.3).abs() < f64::EPSILON);
  }

  #[test]
  fn test_sensor_descriptor_roundtrip() {
    let original = SensorDescriptor {
      id: SensorId::new("gpu.vram.used").unwrap(),
      name: "GPU VRAM Used".to_string(),
      category: "memory".to_string(),
      unit: "bytes".to_string(),
      device: Some("nvidia".to_string()),
      tags: vec!["gpu".to_string(), "vram".to_string()],
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: SensorDescriptor = serde_json::from_str(&json).unwrap();
    assert_eq!(original, parsed);
  }

  #[test]
  fn test_sensor_sample_roundtrip() {
    let original = SensorSample {
      sensor_id: SensorId::new("cpu.core0.frequency").unwrap(),
      timestamp_ms: 1712345678901,
      value: Some(3500.5),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: SensorSample = serde_json::from_str(&json).unwrap();
    assert_eq!(original, parsed);
  }

  #[test]
  fn test_sensor_sample_null_value() {
    let json = r#"{
            "sensor_id": "cpu.temperature",
            "timestamp_ms": 1712345678,
            "value": null
        }"#;
    let sample: SensorSample = serde_json::from_str(json).unwrap();
    assert_eq!(sample.value, None);
  }

  #[test]
  fn test_sensor_sample_null_roundtrip() {
    let original = SensorSample {
      sensor_id: SensorId::new("cpu.temperature").unwrap(),
      timestamp_ms: 1712345678,
      value: None,
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: SensorSample = serde_json::from_str(&json).unwrap();
    assert_eq!(original, parsed);
  }

  #[test]
  fn test_sensor_descriptor_minimal() {
    let json = r#"{
            "id": "cpu.temperature",
            "name": "CPU Temp",
            "category": "temperature",
            "unit": "celsius"
        }"#;
    let descriptor: SensorDescriptor = serde_json::from_str(json).unwrap();
    assert_eq!(descriptor.device, None);
    assert!(descriptor.tags.is_empty());
  }

  #[test]
  fn test_sensor_descriptor_empty_tags() {
    let descriptor = SensorDescriptor {
      id: SensorId::new("cpu.load").unwrap(),
      name: "CPU Load".to_string(),
      category: "utilization".to_string(),
      unit: "percent".to_string(),
      device: None,
      tags: vec![],
    };

    let json = serde_json::to_string(&descriptor).unwrap();
    assert!(!json.contains("device"));
  }
}
