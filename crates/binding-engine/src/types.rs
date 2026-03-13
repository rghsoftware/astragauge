use astragauge_domain::validation::SensorId;
use serde::{Deserialize, Serialize};

/// Transformations that can be applied to raw sensor values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transform {
  /// Round to specified decimal places (u32 = number of places)
  Round(u32),
  /// Restrict value to a range [min, max]
  Clamp { min: f64, max: f64 },
  /// Absolute value
  Abs,
  /// Scale by a factor
  Scale(f64),
  /// Multiply by 100 (semantic alias for percentage)
  Percent,
}

/// Aggregation functions for combining multiple sensor values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aggregation {
  /// Average of all values
  Avg,
  /// Minimum value
  Min,
  /// Maximum value
  Max,
  /// Sum of all values
  Sum,
  /// Number of sensors
  Count,
}

/// Result of resolving a binding after all transforms and aggregations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedBinding {
  /// Final resolved value (None if no data available)
  pub value: Option<f64>,
  /// Number of sensors that contributed to this value
  pub source_count: usize,
}

/// Source of a binding value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BindingSource {
  /// Direct reference to a specific sensor.
  Direct { sensor_id: SensorId },

  /// Wildcard pattern matching multiple sensors with aggregation.
  Wildcard {
    /// Pattern for matching sensor IDs (e.g., "cpu.core*.temperature")
    pattern: String,
    /// How to aggregate values from matching sensors
    aggregation: Aggregation,
  },
}

/// A binding from a sensor source to a widget property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
  /// Source of the binding value
  pub source: BindingSource,

  /// Optional transformation to apply to the value before delivery
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub transform: Option<String>,

  /// Target property on the widget to bind to
  pub target_property: String,
}

/// Errors that can occur during binding operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum BindingError {
  /// Sensor ID could not be resolved
  #[error("unresolved sensor: {0}")]
  UnresolvedSensor(String),

  /// Transform specification is invalid
  #[error("invalid transform: {0}")]
  InvalidTransform(String),

  /// Error occurred during aggregation
  #[error("aggregation error: {0}")]
  AggregationError(String),

  /// Wildcard pattern matched no sensors
  #[error("wildcard pattern matched no sensors: {0}")]
  WildcardNoMatch(String),

  /// Binding ID was not found in the subscription registry
  #[error("binding not found: {0}")]
  BindingNotFound(String),
}

/// Result type for binding operations.
pub type BindingResult<T> = Result<T, BindingError>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn transform_round_serializes() {
    let t = Transform::Round(2);
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("Round"));
    let deserialized: Transform = serde_json::from_str(&json).unwrap();
    assert_eq!(t, deserialized);
  }

  #[test]
  fn transform_clamp_serializes() {
    let t = Transform::Clamp {
      min: 0.0,
      max: 100.0,
    };
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("Clamp"));
    let deserialized: Transform = serde_json::from_str(&json).unwrap();
    assert_eq!(t, deserialized);
  }

  #[test]
  fn aggregation_lowercase_serde() {
    let agg = Aggregation::Avg;
    let json = serde_json::to_string(&agg).unwrap();
    assert_eq!(json, "\"avg\"");

    let deserialized: Aggregation = serde_json::from_str("\"max\"").unwrap();
    assert_eq!(deserialized, Aggregation::Max);
  }

  #[test]
  fn resolved_binding_serializes() {
    let binding = ResolvedBinding {
      value: Some(42.0),
      source_count: 3,
    };
    let json = serde_json::to_string(&binding).unwrap();
    let deserialized: ResolvedBinding = serde_json::from_str(&json).unwrap();
    assert_eq!(binding, deserialized);
  }

  #[test]
  fn test_binding_source_direct_construction() {
    let sensor_id = SensorId::new("cpu.temperature").expect("valid sensor id");
    let source = BindingSource::Direct { sensor_id };

    assert!(matches!(source, BindingSource::Direct { .. }));
  }

  #[test]
  fn test_binding_source_wildcard_construction() {
    let source = BindingSource::Wildcard {
      pattern: "cpu.core*.temperature".to_string(),
      aggregation: Aggregation::Avg,
    };

    assert!(matches!(source, BindingSource::Wildcard { .. }));
    if let BindingSource::Wildcard {
      pattern,
      aggregation,
    } = source
    {
      assert_eq!(pattern, "cpu.core*.temperature");
      assert_eq!(aggregation, Aggregation::Avg);
    }
  }

  #[test]
  fn test_binding_construction() {
    let sensor_id = SensorId::new("cpu.temperature").expect("valid sensor id");
    let source = BindingSource::Direct { sensor_id };

    let binding = Binding {
      source,
      transform: Some("scale(0.5)".to_string()),
      target_property: "value".to_string(),
    };

    assert_eq!(binding.target_property, "value");
    assert_eq!(binding.transform, Some("scale(0.5)".to_string()));
  }

  #[test]
  fn test_binding_without_transform() {
    let sensor_id = SensorId::new("gpu.vram.used").expect("valid sensor id");
    let source = BindingSource::Direct { sensor_id };

    let binding = Binding {
      source,
      transform: None,
      target_property: "percent".to_string(),
    };

    assert_eq!(binding.transform, None);
    assert_eq!(binding.target_property, "percent");
  }

  #[test]
  fn test_binding_error_display() {
    let err = BindingError::UnresolvedSensor("cpu.temperature".to_string());
    assert_eq!(err.to_string(), "unresolved sensor: cpu.temperature");

    let err = BindingError::InvalidTransform("bad syntax".to_string());
    assert_eq!(err.to_string(), "invalid transform: bad syntax");

    let err = BindingError::AggregationError("empty set".to_string());
    assert_eq!(err.to_string(), "aggregation error: empty set");

    let err = BindingError::WildcardNoMatch("cpu.*.temp".to_string());
    assert_eq!(
      err.to_string(),
      "wildcard pattern matched no sensors: cpu.*.temp"
    );

    let err = BindingError::BindingNotFound("my_binding".to_string());
    assert_eq!(err.to_string(), "binding not found: my_binding");
  }

  #[test]
  fn test_binding_result_alias() {
    let result: BindingResult<f64> = Ok(42.0);
    assert!(result.is_ok());

    let result: BindingResult<f64> = Err(BindingError::UnresolvedSensor("test".to_string()));
    assert!(result.is_err());
  }
}
