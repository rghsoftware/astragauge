use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainError {
  InvalidSensorId { id: String, reason: String },
  InvalidFormat { message: String },
  ParseError { message: String },
}

impl fmt::Display for DomainError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DomainError::InvalidSensorId { id, reason } => {
        write!(f, "Invalid sensor ID '{}': {}", id, reason)
      }
      DomainError::InvalidFormat { message } => {
        write!(f, "Invalid format: {}", message)
      }
      DomainError::ParseError { message } => {
        write!(f, "Parse error: {}", message)
      }
    }
  }
}

impl std::error::Error for DomainError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SensorId(String);

impl SensorId {
  pub fn new(id: impl Into<String>) -> Result<Self, DomainError> {
    let id = id.into();

    if id.is_empty() {
      return Err(DomainError::InvalidSensorId {
        id: id.clone(),
        reason: "cannot be empty".to_string(),
      });
    }

    if id.chars().any(|c| c.is_uppercase()) {
      return Err(DomainError::InvalidSensorId {
        id: id.clone(),
        reason: "must be lowercase only".to_string(),
      });
    }

    for c in id.chars() {
      if c != '.' && !c.is_ascii_lowercase() && !c.is_ascii_digit() {
        return Err(DomainError::InvalidSensorId {
          id: id.clone(),
          reason: format!(
            "contains invalid character '{}' (only lowercase letters, digits, and dots allowed)",
            c
          ),
        });
      }
    }

    let segments: Vec<&str> = id.split('.').collect();

    if segments.iter().any(|s| s.is_empty()) {
      return Err(DomainError::InvalidSensorId {
        id: id.clone(),
        reason: "cannot have empty segments (no leading/trailing/consecutive dots)".to_string(),
      });
    }

    let count = segments.len();
    if !(2..=4).contains(&count) {
      return Err(DomainError::InvalidSensorId {
        id: id.clone(),
        reason: format!("must have 2-4 segments, found {}", count),
      });
    }

    Ok(SensorId(id))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for SensorId {
  type Err = DomainError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::new(s)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_sensor_ids() {
    assert!(SensorId::new("cpu.temperature").is_ok());
    assert!(SensorId::new("gpu.utilization").is_ok());
    assert!(SensorId::new("cpu.core0.frequency").is_ok());
    assert!(SensorId::new("gpu.vram.controller.temperature").is_ok());
  }

  #[test]
  fn test_invalid_uppercase() {
    let result = SensorId::new("CPU.Temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("lowercase"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_underscore() {
    let result = SensorId::new("cpu_temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("invalid character"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_empty() {
    let result = SensorId::new("");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("empty"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_consecutive_dots() {
    let result = SensorId::new("cpu..temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("empty segments"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_leading_dot() {
    let result = SensorId::new(".temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("empty segments"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_trailing_dot() {
    let result = SensorId::new("cpu.");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("empty segments"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_too_few_segments() {
    let result = SensorId::new("cpu");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("2-4 segments"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_too_many_segments() {
    let result = SensorId::new("cpu.core0.0.frequency.temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("2-4 segments"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_hyphen() {
    let result = SensorId::new("cpu-core-0.temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("invalid character"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_space() {
    let result = SensorId::new("cpu core.temperature");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("invalid character"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_from_str() {
    use std::str::FromStr;
    assert!(SensorId::from_str("cpu.temperature").is_ok());
    assert!(SensorId::from_str("CPU.Temperature").is_err());
  }

  #[test]
  fn test_as_str() {
    let id = SensorId::new("cpu.temperature").unwrap();
    assert_eq!(id.as_str(), "cpu.temperature");
  }

  #[test]
  fn test_display_error() {
    let err = DomainError::InvalidSensorId {
      id: "CPU.Temperature".to_string(),
      reason: "must be lowercase only".to_string(),
    };
    let display = format!("{}", err);
    assert!(display.contains("CPU.Temperature"));
    assert!(display.contains("lowercase"));
  }

  #[test]
  fn test_invalid_unicode() {
    let result = SensorId::new("cpu.température");
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidSensorId { reason, .. }) => {
        assert!(reason.contains("invalid character"));
      }
      _ => panic!("Expected InvalidSensorId error"),
    }
  }

  #[test]
  fn test_invalid_emoji() {
    let result = SensorId::new("cpu.🔥");
    assert!(result.is_err());
  }

  #[test]
  fn test_invalid_cyrillic() {
    let result = SensorId::new("cpu.температура");
    assert!(result.is_err());
  }

  #[test]
  fn test_error_parse_variant() {
    let err = DomainError::ParseError {
      message: "failed to parse".to_string(),
    };
    let display = format!("{}", err);
    assert!(display.contains("Parse error"));
    assert!(display.contains("failed to parse"));
  }

  #[test]
  fn test_error_invalid_format_variant() {
    let err = DomainError::InvalidFormat {
      message: "expected JSON".to_string(),
    };
    let display = format!("{}", err);
    assert!(display.contains("Invalid format"));
    assert!(display.contains("expected JSON"));
  }
}
