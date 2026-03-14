// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use astragauge_provider_host::{HostConfig, ProviderHost};
use astragauge_sensor_store::SensorStore;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let store = Arc::new(SensorStore::new());
  let config = HostConfig::default();
  let host = Arc::new(ProviderHost::new(config, store));

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .manage(host)
    .invoke_handler(tauri::generate_handler![greet, get_providers_status])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
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
}
