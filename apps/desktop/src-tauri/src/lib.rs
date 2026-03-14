// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use astragauge_provider_host::{HostConfig, ProviderHost};
use astragauge_sensor_store::SensorStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_providers_status(
  host: State<Arc<ProviderHost>>,
) -> Vec<astragauge_provider_host::ProviderStatus> {
  host.get_providers_status()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorInfo {
  pub id: String,
  pub name: String,
  pub category: String,
  pub unit: String,
  pub provider_id: String,
}

#[tauri::command]
async fn list_available_sensors(
  store: State<'_, Arc<SensorStore>>,
) -> Result<Vec<SensorInfo>, String> {
  let sensor_ids = store.list_sensors().await;

  let mut sensors = Vec::new();
  for sensor_id in sensor_ids {
    if let Some(descriptor) = store.get_descriptor(&sensor_id).await {
      let provider_id = sensor_id
        .split('.')
        .next()
        .unwrap_or(&sensor_id)
        .to_string();

      sensors.push(SensorInfo {
        id: descriptor.id.to_string(),
        name: descriptor.name,
        category: descriptor.category,
        unit: descriptor.unit,
        provider_id,
      });
    }
  }

  Ok(sensors)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = Arc::new(ProviderHost::new(config, store));

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .manage(host)
    .invoke_handler(tauri::generate_handler![
      greet,
      get_providers_status,
      list_available_sensors
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use super::*;
  use astragauge_provider_host::{ProviderHealth, ProviderStatus};

  #[test]
  fn test_provider_status_serialization() {
    let status = ProviderStatus {
      id: "test-provider".to_string(),
      name: "Test Provider".to_string(),
      health: ProviderHealth::Ok,
      sensor_count: 5,
    };

    let json = serde_json::to_string(&status).expect("Should serialize to JSON");
    assert!(json.contains("\"id\":\"test-provider\""));
    assert!(json.contains("\"name\":\"Test Provider\""));
    assert!(json.contains("\"health\":\"ok\""));
    assert!(json.contains("\"sensor_count\":5"));
  }

  #[test]
  fn test_empty_provider_status_list() {
    let statuses: Vec<ProviderStatus> = vec![];
    let json = serde_json::to_string(&statuses).expect("Should serialize empty list");
    assert_eq!(json, "[]");
  }

  #[test]
  fn test_provider_health_serialization() {
    let health = ProviderHealth::Ok;
    let json = serde_json::to_string(&health).expect("Should serialize to JSON");
    assert_eq!(json, "\"ok\"");

    let degraded = ProviderHealth::Degraded {
      message: "Test degradation".to_string(),
    };
    let json = serde_json::to_string(&degraded).expect("Should serialize to JSON");
    assert!(json.contains("degraded"));
    assert!(json.contains("Test degradation"));

    let error = ProviderHealth::Error {
      message: "Test error".to_string(),
    };
    let json = serde_json::to_string(&error).expect("Should serialize to JSON");
    assert!(json.contains("error"));
    assert!(json.contains("Test error"));
  }

  #[test]
  fn test_sensor_info_serialization() {
    let sensor = SensorInfo {
      id: "cpu.temperature".to_string(),
      name: "CPU Temperature".to_string(),
      category: "temperature".to_string(),
      unit: "celsius".to_string(),
      provider_id: "cpu".to_string(),
    };

    let json = serde_json::to_string(&sensor).expect("Should serialize to JSON");
    assert!(json.contains("\"id\":\"cpu.temperature\""));
    assert!(json.contains("\"name\":\"CPU Temperature\""));
    assert!(json.contains("\"category\":\"temperature\""));
    assert!(json.contains("\"unit\":\"celsius\""));
    assert!(json.contains("\"provider_id\":\"cpu\""));
  }

  #[test]
  fn test_empty_sensor_list() {
    let sensors: Vec<SensorInfo> = vec![];
    let json = serde_json::to_string(&sensors).expect("Should serialize empty list");
    assert_eq!(json, "[]");
  }
}
