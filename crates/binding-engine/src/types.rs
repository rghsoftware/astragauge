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
  /// Celsius to Fahrenheit: F = C * 9/5 + 32
  CelsiusToFahrenheit,
  /// Bytes to kilobytes (÷1024)
  BytesToKb,
  /// Bytes to megabytes (÷1024²)
  BytesToMb,
  /// Bytes to gigabytes (÷1024³)
  BytesToGb,
  /// Bytes to terabytes (÷1024⁴)
  BytesToTb,
  /// Bits to kilobits (÷1000)
  BitsToKbit,
  /// Bits to megabits (÷1000²)
  BitsToMbit,
  /// Bits to gigabits (÷1000³)
  BitsToGbit,
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

/// Specification for formatting a resolved binding value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatSpec {
  /// Number of decimal places (0 = integer display)
  #[serde(default)]
  pub decimal_places: u32,
  /// Optional unit suffix appended to formatted value (e.g., "°C", " MB")
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub unit_suffix: Option<String>,
  /// String to display when value is None
  #[serde(default = "default_na_string")]
  pub na_string: String,
}

fn default_na_string() -> String {
  "N/A".to_string()
}

impl Default for FormatSpec {
  fn default() -> Self {
    Self {
      decimal_places: 2,
      unit_suffix: None,
      na_string: default_na_string(),
    }
  }
}

/// A resolved binding with a formatted display string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattedBinding {
  /// The raw resolved value (None if unavailable)
  pub raw_value: Option<f64>,
  /// Human-readable formatted string
  pub formatted_value: String,
  /// Number of sensors that contributed
  pub source_count: usize,
}

/// Formats a resolved binding according to a format specification.
pub fn format_value(resolved: &ResolvedBinding, spec: &FormatSpec) -> FormattedBinding {
  let formatted_value = match resolved.value {
    Some(v) => {
      let formatted = format!("{:.1$}", v, spec.decimal_places as usize);
      match &spec.unit_suffix {
        Some(suffix) => format!("{}{}", formatted, suffix),
        None => formatted,
      }
    }
    None => spec.na_string.clone(),
  };

  FormattedBinding {
    raw_value: resolved.value,
    formatted_value,
    source_count: resolved.source_count,
  }
}

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

  // ===== FORMAT SPEC =====

  #[test]
  fn test_format_spec_default() {
    let spec = FormatSpec::default();
    assert_eq!(spec.decimal_places, 2);
    assert_eq!(spec.unit_suffix, None);
    assert_eq!(spec.na_string, "N/A");
  }

  #[test]
  fn test_format_value_with_suffix() {
    let resolved = ResolvedBinding {
      value: Some(42.567),
      source_count: 1,
    };
    let spec = FormatSpec {
      decimal_places: 1,
      unit_suffix: Some("°C".to_string()),
      na_string: "N/A".to_string(),
    };
    let formatted = format_value(&resolved, &spec);
    assert_eq!(formatted.formatted_value, "42.6°C");
    assert_eq!(formatted.raw_value, Some(42.567));
    assert_eq!(formatted.source_count, 1);
  }

  #[test]
  fn test_format_value_none() {
    let resolved = ResolvedBinding {
      value: None,
      source_count: 0,
    };
    let spec = FormatSpec::default();
    let formatted = format_value(&resolved, &spec);
    assert_eq!(formatted.formatted_value, "N/A");
  }

  #[test]
  fn test_format_value_integer() {
    let resolved = ResolvedBinding {
      value: Some(100.0),
      source_count: 1,
    };
    let spec = FormatSpec {
      decimal_places: 0,
      unit_suffix: Some("%".to_string()),
      na_string: "---".to_string(),
    };
    let formatted = format_value(&resolved, &spec);
    assert_eq!(formatted.formatted_value, "100%");
  }

  #[test]
  fn test_format_value_no_suffix() {
    let resolved = ResolvedBinding {
      value: Some(3.14159),
      source_count: 1,
    };
    let spec = FormatSpec {
      decimal_places: 3,
      unit_suffix: None,
      na_string: "N/A".to_string(),
    };
    let formatted = format_value(&resolved, &spec);
    assert_eq!(formatted.formatted_value, "3.142");
  }

  #[test]
  fn test_format_value_custom_na_string() {
    let resolved = ResolvedBinding {
      value: None,
      source_count: 0,
    };
    let spec = FormatSpec {
      decimal_places: 2,
      unit_suffix: None,
      na_string: "---".to_string(),
    };
    let formatted = format_value(&resolved, &spec);
    assert_eq!(formatted.formatted_value, "---");
  }

  #[test]
  fn test_format_spec_serialization() {
    let spec = FormatSpec {
      decimal_places: 1,
      unit_suffix: Some(" MB".to_string()),
      na_string: "N/A".to_string(),
    };
    let json = serde_json::to_string(&spec).unwrap();
    let deserialized: FormatSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(spec, deserialized);
  }

  #[test]
  fn test_formatted_binding_serialization() {
    let fb = FormattedBinding {
      raw_value: Some(42.5),
      formatted_value: "42.5°C".to_string(),
      source_count: 1,
    };
    let json = serde_json::to_string(&fb).unwrap();
    let deserialized: FormattedBinding = serde_json::from_str(&json).unwrap();
    assert_eq!(fb, deserialized);
  }
}
