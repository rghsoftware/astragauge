use serde::{Deserialize, Serialize};

use crate::validation::DomainError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderCapabilities {
  pub historical: bool,
  pub high_frequency: bool,
  pub hardware_access: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SensorCategories {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub categories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderManifest {
  pub id: String,
  pub name: String,
  pub version: String,
  pub description: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub author: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub website: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub repository: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub license: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tags: Option<Vec<String>>,
  pub runtime: String,
  pub capabilities: ProviderCapabilities,
  pub sensors: SensorCategories,
}

impl ProviderManifest {
  /// Validates the manifest for required fields and basic format rules.
  /// Returns Ok(()) if valid, Err(DomainError) if invalid.
  pub fn validate(&self) -> Result<(), DomainError> {
    if self.id.is_empty() {
      return Err(DomainError::InvalidFormat {
        message: "provider id cannot be empty".to_string(),
      });
    }

    if self.version.is_empty() {
      return Err(DomainError::InvalidFormat {
        message: "provider version cannot be empty".to_string(),
      });
    }

    if self.runtime.is_empty() {
      return Err(DomainError::InvalidFormat {
        message: "provider runtime requirement cannot be empty".to_string(),
      });
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_provider_toml() {
    let toml_str = r#"
id = "core.linux"
name = "Linux System Provider"
version = "0.1.0"
description = "Collects system metrics from Linux using /proc and hwmon."
author = "AstraGauge Project"
runtime = ">=0.1.0"

[capabilities]
historical = false
high_frequency = true
hardware_access = true

[sensors]
categories = ["cpu","memory","disk","network","temperature"]
"#;

    let manifest: ProviderManifest = toml::from_str(toml_str).unwrap();

    assert_eq!(manifest.id, "core.linux");
    assert_eq!(manifest.name, "Linux System Provider");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(
      manifest.description,
      "Collects system metrics from Linux using /proc and hwmon."
    );
    assert_eq!(manifest.author, Some("AstraGauge Project".to_string()));
    assert_eq!(manifest.runtime, ">=0.1.0");
    assert!(!manifest.capabilities.historical);
    assert!(manifest.capabilities.high_frequency);
    assert!(manifest.capabilities.hardware_access);
    assert_eq!(
      manifest.sensors.categories,
      vec!["cpu", "memory", "disk", "network", "temperature"]
    );
  }

  #[test]
  fn test_provider_json() {
    let json_str = r#"
{
  "id": "core.linux",
  "name": "Linux System Provider",
  "version": "0.1.0",
  "description": "Collects system metrics from Linux using /proc and hwmon.",
  "author": "AstraGauge Project",
  "runtime": ">=0.1.0",
  "capabilities": {
    "historical": false,
    "high_frequency": true,
    "hardware_access": true
  },
  "sensors": {
    "categories": ["cpu", "memory", "disk", "network", "temperature"]
  }
}
"#;

    let manifest: ProviderManifest = serde_json::from_str(json_str).unwrap();

    assert_eq!(manifest.id, "core.linux");
    assert_eq!(manifest.name, "Linux System Provider");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(
      manifest.description,
      "Collects system metrics from Linux using /proc and hwmon."
    );
    assert_eq!(manifest.author, Some("AstraGauge Project".to_string()));
    assert_eq!(manifest.runtime, ">=0.1.0");
    assert!(!manifest.capabilities.historical);
    assert!(manifest.capabilities.high_frequency);
    assert!(manifest.capabilities.hardware_access);
    assert_eq!(
      manifest.sensors.categories,
      vec!["cpu", "memory", "disk", "network", "temperature"]
    );
  }

  #[test]
  fn test_provider_roundtrip() {
    let original = ProviderManifest {
      id: "core.linux".to_string(),
      name: "Linux System Provider".to_string(),
      version: "0.1.0".to_string(),
      description: "Collects system metrics from Linux using /proc and hwmon.".to_string(),
      author: Some("AstraGauge Project".to_string()),
      website: Some("https://astragauge.dev".to_string()),
      repository: Some("https://github.com/rghsoftware/astragauge".to_string()),
      license: Some("MIT".to_string()),
      tags: Some(vec!["system".to_string(), "linux".to_string()]),
      runtime: ">=0.1.0".to_string(),
      capabilities: ProviderCapabilities {
        historical: false,
        high_frequency: true,
        hardware_access: true,
      },
      sensors: SensorCategories {
        categories: vec!["cpu", "memory", "disk", "network", "temperature"]
          .into_iter()
          .map(String::from)
          .collect(),
      },
    };

    let toml_str = toml::to_string_pretty(&original).unwrap();
    let from_toml: ProviderManifest = toml::from_str(&toml_str).unwrap();
    assert_eq!(original, from_toml);

    let json_str = serde_json::to_string_pretty(&original).unwrap();
    let from_json: ProviderManifest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(original, from_json);
  }

  #[test]
  fn test_provider_minimal() {
    let json = r#"{
            "id": "minimal",
            "name": "Minimal Provider",
            "version": "1.0.0",
            "description": "A minimal provider",
            "runtime": ">=1.0.0",
            "capabilities": {
                "historical": false,
                "high_frequency": false,
                "hardware_access": false
            },
            "sensors": {}
        }"#;

    let manifest: ProviderManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.author, None);
    assert_eq!(manifest.tags, None);
    assert!(manifest.sensors.categories.is_empty());
  }

  #[test]
  fn test_sensor_categories_empty() {
    let categories = SensorCategories { categories: vec![] };
    assert!(categories.categories.is_empty());

    let json = serde_json::to_string(&categories).unwrap();
    let parsed: SensorCategories = serde_json::from_str(&json).unwrap();
    assert_eq!(categories, parsed);
  }

  #[test]
  fn test_validate_valid_manifest() {
    let manifest = ProviderManifest {
      id: "core.linux".to_string(),
      name: "Linux System Provider".to_string(),
      version: "0.1.0".to_string(),
      description: "Test provider".to_string(),
      author: None,
      website: None,
      repository: None,
      license: None,
      tags: None,
      runtime: ">=0.1.0".to_string(),
      capabilities: ProviderCapabilities {
        historical: false,
        high_frequency: true,
        hardware_access: true,
      },
      sensors: SensorCategories { categories: vec![] },
    };

    assert!(manifest.validate().is_ok());
  }

  #[test]
  fn test_validate_empty_id() {
    let manifest = ProviderManifest {
      id: "".to_string(),
      name: "Test Provider".to_string(),
      version: "0.1.0".to_string(),
      description: "Test provider".to_string(),
      author: None,
      website: None,
      repository: None,
      license: None,
      tags: None,
      runtime: ">=0.1.0".to_string(),
      capabilities: ProviderCapabilities {
        historical: false,
        high_frequency: false,
        hardware_access: false,
      },
      sensors: SensorCategories { categories: vec![] },
    };

    let result = manifest.validate();
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidFormat { message }) => {
        assert!(message.contains("id cannot be empty"));
      }
      _ => panic!("Expected InvalidFormat error for empty id"),
    }
  }

  #[test]
  fn test_validate_empty_version() {
    let manifest = ProviderManifest {
      id: "test.provider".to_string(),
      name: "Test Provider".to_string(),
      version: "".to_string(),
      description: "Test provider".to_string(),
      author: None,
      website: None,
      repository: None,
      license: None,
      tags: None,
      runtime: ">=0.1.0".to_string(),
      capabilities: ProviderCapabilities {
        historical: false,
        high_frequency: false,
        hardware_access: false,
      },
      sensors: SensorCategories { categories: vec![] },
    };

    let result = manifest.validate();
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidFormat { message }) => {
        assert!(message.contains("version cannot be empty"));
      }
      _ => panic!("Expected InvalidFormat error for empty version"),
    }
  }

  #[test]
  fn test_validate_empty_runtime() {
    let manifest = ProviderManifest {
      id: "test.provider".to_string(),
      name: "Test Provider".to_string(),
      version: "0.1.0".to_string(),
      description: "Test provider".to_string(),
      author: None,
      website: None,
      repository: None,
      license: None,
      tags: None,
      runtime: "".to_string(),
      capabilities: ProviderCapabilities {
        historical: false,
        high_frequency: false,
        hardware_access: false,
      },
      sensors: SensorCategories { categories: vec![] },
    };

    let result = manifest.validate();
    assert!(result.is_err());
    match result {
      Err(DomainError::InvalidFormat { message }) => {
        assert!(message.contains("runtime requirement cannot be empty"));
      }
      _ => panic!("Expected InvalidFormat error for empty runtime"),
    }
  }
}
