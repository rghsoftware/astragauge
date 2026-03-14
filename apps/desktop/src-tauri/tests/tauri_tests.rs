//! Integration tests for Tauri commands.
//!
//! These tests verify the JSON structure and behavior of Tauri commands
//! without starting the full Tauri application.

use std::sync::Arc;

use astragauge_domain::{SensorDescriptor, SensorId};
use astragauge_provider_host::{
  HostConfig, Provider, ProviderHealth, ProviderHost, ProviderStatus,
};
use astragauge_providers::MockProvider;
use astragauge_sensor_store::SensorStore;
use serde::{Deserialize, Serialize};

/// SensorInfo mirrors the structure in lib.rs for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorInfo {
  pub id: String,
  pub name: String,
  pub category: String,
  pub unit: String,
  pub device_id: String,
}

/// Helper function that replicates the list_available_sensors command logic
/// for testing without the Tauri State wrapper.
async fn list_available_sensors_logic(store: &Arc<SensorStore>) -> Result<Vec<SensorInfo>, String> {
  let sensor_ids = store.list_sensors().await;

  let mut sensors = Vec::new();
  for sensor_id in sensor_ids {
    if let Some(descriptor) = store.get_descriptor(&sensor_id).await {
      let device_id = sensor_id
        .split('.')
        .next()
        .unwrap_or(&sensor_id)
        .to_string();

      sensors.push(SensorInfo {
        id: descriptor.id.to_string(),
        name: descriptor.name,
        category: descriptor.category,
        unit: descriptor.unit,
        device_id,
      });
    }
  }

  Ok(sensors)
}

// ============================================================================
// Test 1: get_providers_status returns valid structure
// ============================================================================

#[test]
fn get_providers_status_returns_valid_structure() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = Arc::new(ProviderHost::new(config, store));

  // Call the method that the Tauri command wraps
  let statuses: Vec<ProviderStatus> = host.get_providers_status();

  // Empty host should return empty list
  assert!(
    statuses.is_empty(),
    "Expected empty provider list for new host"
  );

  // Verify the structure can be serialized to JSON with expected fields
  let json = serde_json::to_string(&statuses).expect("Should serialize to JSON");
  assert_eq!(
    json, "[]",
    "Empty provider list should serialize to empty JSON array"
  );
}

#[test]
fn get_providers_status_with_provider_returns_valid_structure() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let mut host = ProviderHost::new(config, store);

  // Register a mock provider
  let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::new_test()));
  host
    .register_provider(provider)
    .expect("Should register provider");

  // Now check status
  let statuses = host.get_providers_status();
  assert_eq!(statuses.len(), 1, "Expected one provider in status list");

  let status = &statuses[0];
  assert_eq!(status.id, "mock.provider");
  assert_eq!(status.name, "Mock Provider");
  assert!(matches!(status.health, ProviderHealth::Ok));

  // Verify JSON structure contains all expected fields
  let json = serde_json::to_string(&status).expect("Should serialize to JSON");
  assert!(json.contains("\"id\""), "JSON should contain 'id' field");
  assert!(
    json.contains("\"name\""),
    "JSON should contain 'name' field"
  );
  assert!(
    json.contains("\"health\""),
    "JSON should contain 'health' field"
  );
  assert!(
    json.contains("\"sensor_count\""),
    "JSON should contain 'sensor_count' field"
  );
}

// ============================================================================
// Test 2: list_available_sensors returns valid structure
// ============================================================================

#[tokio::test]
async fn list_available_sensors_returns_valid_structure() {
  let store = Arc::new(SensorStore::new());

  // Register a sensor directly in the store
  let sensor_id = SensorId::new("test.temperature").expect("valid sensor id");
  let descriptor = SensorDescriptor {
    id: sensor_id.clone(),
    name: "Test Temperature".to_string(),
    category: "temperature".to_string(),
    unit: "celsius".to_string(),
    device: None,
    tags: vec![],
  };

  store
    .register_sensor(descriptor)
    .await
    .expect("Should register sensor");

  // Call the logic function
  let sensors = list_available_sensors_logic(&store)
    .await
    .expect("Should return sensors");

  assert_eq!(sensors.len(), 1, "Expected one sensor in list");

  let sensor = &sensors[0];
  assert_eq!(sensor.id, "test.temperature");
  assert_eq!(sensor.name, "Test Temperature");
  assert_eq!(sensor.category, "temperature");
  assert_eq!(sensor.unit, "celsius");
  assert_eq!(sensor.device_id, "test");

  // Verify JSON structure
  let json = serde_json::to_string(&sensors).expect("Should serialize to JSON");
  assert!(
    json.contains("\"id\":\"test.temperature\""),
    "JSON should contain sensor id"
  );
  assert!(
    json.contains("\"name\":\"Test Temperature\""),
    "JSON should contain sensor name"
  );
  assert!(
    json.contains("\"category\":\"temperature\""),
    "JSON should contain category"
  );
  assert!(
    json.contains("\"unit\":\"celsius\""),
    "JSON should contain unit"
  );
  assert!(
    json.contains("\"device_id\":\"test\""),
    "JSON should contain device_id"
  );
}

#[tokio::test]
async fn list_available_sensors_extracts_device_id_correctly() {
  let store = Arc::new(SensorStore::new());

  // Register sensors with different provider prefixes
  let sensors_to_register = vec![
    (
      "cpu.core0.temperature",
      "CPU Core 0 Temp",
      "temperature",
      "celsius",
    ),
    ("gpu.vram.used", "GPU VRAM Used", "memory", "bytes"),
    ("system.uptime", "System Uptime", "time", "seconds"),
  ];

  for (id_str, name, category, unit) in sensors_to_register {
    let sensor_id = SensorId::new(id_str).expect("valid sensor id");
    let descriptor = SensorDescriptor {
      id: sensor_id,
      name: name.to_string(),
      category: category.to_string(),
      unit: unit.to_string(),
      device: None,
      tags: vec![],
    };
    store
      .register_sensor(descriptor)
      .await
      .expect("Should register sensor");
  }

  let sensors = list_available_sensors_logic(&store)
    .await
    .expect("Should return sensors");

  assert_eq!(sensors.len(), 3);

  // Verify device_id extraction (first component before dot)
  let cpu_sensor = sensors
    .iter()
    .find(|s| s.id == "cpu.core0.temperature")
    .unwrap();
  assert_eq!(cpu_sensor.device_id, "cpu");

  let gpu_sensor = sensors.iter().find(|s| s.id == "gpu.vram.used").unwrap();
  assert_eq!(gpu_sensor.device_id, "gpu");

  let system_sensor = sensors.iter().find(|s| s.id == "system.uptime").unwrap();
  assert_eq!(system_sensor.device_id, "system");
}

// ============================================================================
// Test 3: commands work without providers loaded (empty state)
// ============================================================================

#[test]
fn commands_work_without_providers_loaded() {
  // Test get_providers_status with empty host
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = Arc::new(ProviderHost::new(config, store));

  let statuses = host.get_providers_status();
  assert!(
    statuses.is_empty(),
    "Empty host should return empty status list"
  );

  // Verify empty list serializes correctly
  let json = serde_json::to_string(&statuses).expect("Should serialize");
  assert_eq!(json, "[]");
}

#[tokio::test]
async fn list_available_sensors_empty_state() {
  // Test list_available_sensors with empty store
  let store = Arc::new(SensorStore::new());

  let sensors = list_available_sensors_logic(&store)
    .await
    .expect("Should return sensors");
  assert!(
    sensors.is_empty(),
    "Empty store should return empty sensor list"
  );

  // Verify empty list serializes correctly
  let json = serde_json::to_string(&sensors).expect("Should serialize");
  assert_eq!(json, "[]");
}

#[tokio::test]
async fn full_integration_empty_state() {
  // Combined test: verify both commands work with empty ProviderHost
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = Arc::new(ProviderHost::new(config, Arc::clone(&store)));

  // Test get_providers_status
  let statuses = host.get_providers_status();
  assert!(statuses.is_empty());

  // Test list_available_sensors
  let sensors = list_available_sensors_logic(&store)
    .await
    .expect("Should return sensors");
  assert!(sensors.is_empty());

  // Verify both can be serialized for Tauri IPC
  let status_json = serde_json::to_string(&statuses).expect("Should serialize");
  let sensor_json = serde_json::to_string(&sensors).expect("Should serialize");

  assert_eq!(status_json, "[]");
  assert_eq!(sensor_json, "[]");
}

// ============================================================================
// Additional structure validation tests
// ============================================================================

#[test]
fn provider_status_health_variants_serialize_correctly() {
  // Test Ok variant
  let ok_health = ProviderHealth::Ok;
  let json = serde_json::to_string(&ok_health).expect("Should serialize");
  assert_eq!(json, "\"ok\"");

  // Test Degraded variant
  let degraded = ProviderHealth::Degraded {
    message: "Performance issues".to_string(),
  };
  let json = serde_json::to_string(&degraded).expect("Should serialize");
  assert!(json.contains("degraded"));
  assert!(json.contains("Performance issues"));

  // Test Error variant
  let error = ProviderHealth::Error {
    message: "Connection failed".to_string(),
  };
  let json = serde_json::to_string(&error).expect("Should serialize");
  assert!(json.contains("error"));
  assert!(json.contains("Connection failed"));
}

#[test]
fn sensor_info_serializes_to_expected_json_format() {
  let sensor = SensorInfo {
    id: "cpu.temperature".to_string(),
    name: "CPU Temperature".to_string(),
    category: "temperature".to_string(),
    unit: "celsius".to_string(),
    device_id: "cpu".to_string(),
  };

  let json = serde_json::to_string(&sensor).expect("Should serialize");

  // Verify all fields are present with correct values
  let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should parse JSON");

  assert_eq!(parsed["id"], "cpu.temperature");
  assert_eq!(parsed["name"], "CPU Temperature");
  assert_eq!(parsed["category"], "temperature");
  assert_eq!(parsed["unit"], "celsius");
  assert_eq!(parsed["device_id"], "cpu");
}
