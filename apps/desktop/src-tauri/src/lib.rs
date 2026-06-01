// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};

use astragauge_provider_host::{HostConfig, ProviderHost};
use astragauge_providers::MockProvider;
use astragauge_sensor_store::SensorStore;
use std::sync::Arc;
use tauri::{Emitter, State};

/// Live sensor reading emitted to the frontend. Field names are serialized
/// as camelCase to match the TypeScript `SensorReading` interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReading {
  pub id: String,
  pub name: String,
  pub category: String,
  pub unit: String,
  pub value: Option<f64>,
  #[serde(rename = "timestampMs")]
  pub timestamp_ms: u64,
}

/// Builds a snapshot of all sensors currently registered in the store.
/// Shared by the `get_sensor_snapshot` command and the background emitter
/// so both produce an identical payload.
async fn build_snapshot(store: &SensorStore) -> Vec<SensorReading> {
  let sensor_ids = store.list_sensors().await;

  let mut readings = Vec::new();
  for sensor_id in sensor_ids {
    if let Some(descriptor) = store.get_descriptor(&sensor_id).await {
      let (value, timestamp_ms) = match store.get_value_with_timestamp(&sensor_id).await {
        Some((sample, ts)) => (sample.value, ts),
        None => (None, 0),
      };

      readings.push(SensorReading {
        id: descriptor.id.to_string(),
        name: descriptor.name,
        category: descriptor.category,
        unit: descriptor.unit,
        value,
        timestamp_ms,
      });
    } else {
      tracing::warn!(
        "Sensor {:?} has no descriptor; omitting from snapshot",
        sensor_id
      );
    }
  }

  readings
}

#[tauri::command]
async fn get_sensor_snapshot(
  store: State<'_, Arc<SensorStore>>,
) -> Result<Vec<SensorReading>, String> {
  Ok(build_snapshot(&store).await)
}

#[tauri::command]
fn get_providers_status(
  host: State<Arc<std::sync::RwLock<ProviderHost>>>,
) -> Result<Vec<astragauge_provider_host::ProviderStatus>, String> {
  host
    .read()
    .map(|h| h.get_providers_status())
    .map_err(|e| format!("Failed to acquire provider host lock: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorInfo {
  pub id: String,
  pub name: String,
  pub category: String,
  pub unit: String,
  pub device_id: String,
}

#[tauri::command]
async fn list_available_sensors(
  store: State<'_, Arc<SensorStore>>,
) -> Result<Vec<SensorInfo>, String> {
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

/// Builds the demo MockProvider, which emits the canonical sensor ids the
/// default panel binds to (so the UI stays populated even as a fallback).
fn make_mock_provider() -> Arc<Box<dyn astragauge_provider_host::Provider>> {
  Arc::new(Box::new(MockProvider::new_demo()))
}

/// Selects the sensor provider based on the `ASTRAGAUGE_PROVIDER` env var.
///
/// Values: `auto` (default) | `mock` | `linux`.
/// - `auto`: on Linux, use LinuxProvider unless it discovers 0 sensors, in
///   which case fall back to the demo MockProvider. On non-Linux, use mock.
/// - `mock`: always the demo MockProvider.
/// - `linux`: force LinuxProvider on Linux; on non-Linux, warn and use mock.
///
/// Returns the chosen provider plus a human-readable reason for logging.
/// Every reference to `LinuxProvider` is `#[cfg(target_os = "linux")]`-gated
/// so the app still compiles on macOS/Windows.
fn select_provider() -> (Arc<Box<dyn astragauge_provider_host::Provider>>, String) {
  let requested = std::env::var("ASTRAGAUGE_PROVIDER").unwrap_or_default();
  let mode = requested.trim().to_ascii_lowercase();

  match mode.as_str() {
    "mock" => (
      make_mock_provider(),
      "MockProvider (demo): ASTRAGAUGE_PROVIDER=mock".to_string(),
    ),
    "linux" => {
      #[cfg(target_os = "linux")]
      {
        let provider = astragauge_providers::LinuxProvider::new();
        (
          Arc::new(Box::new(provider)),
          "LinuxProvider: ASTRAGAUGE_PROVIDER=linux (forced)".to_string(),
        )
      }
      #[cfg(not(target_os = "linux"))]
      {
        tracing::warn!(
          "ASTRAGAUGE_PROVIDER=linux requested on a non-Linux platform; \
           LinuxProvider is unavailable, using demo MockProvider instead"
        );
        (
          make_mock_provider(),
          "MockProvider (demo): ASTRAGAUGE_PROVIDER=linux requested on non-Linux platform"
            .to_string(),
        )
      }
    }
    // "auto", empty, or any unrecognized value -> auto behavior.
    other => {
      if !other.is_empty() && other != "auto" {
        tracing::warn!(
          "Unrecognized ASTRAGAUGE_PROVIDER={:?}; defaulting to auto selection",
          other
        );
      }

      #[cfg(target_os = "linux")]
      {
        let provider = astragauge_providers::LinuxProvider::new();
        // discover() returns the eagerly-discovered descriptors; a clone of a
        // Vec, so this resolves immediately on Tauri's global runtime.
        let sensor_count =
          tauri::async_runtime::block_on(astragauge_provider_host::Provider::discover(&provider))
            .map(|sensors| sensors.len())
            .unwrap_or(0);

        if sensor_count > 0 {
          (
            Arc::new(Box::new(provider)),
            format!(
              "LinuxProvider: auto selected ({} sensors discovered)",
              sensor_count
            ),
          )
        } else {
          (
            make_mock_provider(),
            "MockProvider (demo): auto fallback — LinuxProvider discovered 0 sensors".to_string(),
          )
        }
      }
      #[cfg(not(target_os = "linux"))]
      {
        (
          make_mock_provider(),
          "MockProvider (demo): auto selection on non-Linux platform".to_string(),
        )
      }
    }
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let store = Arc::new(SensorStore::new());
  let store_clone = Arc::clone(&store);
  let config = HostConfig::default();
  let host = Arc::new(std::sync::RwLock::new(ProviderHost::new(config, store)));

  let (provider, reason) = select_provider();
  tracing::info!("Selected sensor provider: {}", reason);
  host
    .write()
    .expect("Provider host lock should not be poisoned at startup")
    .register_provider(provider)
    .expect("Failed to register selected provider");

  let host_clone = Arc::clone(&host);
  let store_for_emitter = Arc::clone(&store_clone);

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .manage(host)
    .manage(store_clone)
    .invoke_handler(tauri::generate_handler![
      get_providers_status,
      list_available_sensors,
      get_sensor_snapshot
    ])
    .setup(move |app| {
      tauri::async_runtime::spawn(async move {
        match host_clone.write() {
          Ok(mut host) => {
            host.start();
          }
          Err(e) => {
            tracing::error!("Provider host lock poisoned: {}", e);
          }
        }
      });

      let app_handle = app.handle().clone();
      // Assigned to suppress the unused-must-use warning. Dropping a tokio
      // JoinHandle detaches the task; it keeps running.
      let _emitter_handle = tauri::async_runtime::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(500));
        loop {
          tick.tick().await;
          let store_ref = Arc::clone(&store_for_emitter);
          // Spawn a child task so a panic in build_snapshot is caught via
          // JoinError instead of killing the emitter loop.
          match tokio::spawn(async move { build_snapshot(&store_ref).await }).await {
            Ok(readings) => {
              if let Err(e) = app_handle.emit("sensors-update", &readings) {
                tracing::error!("Failed to emit sensors-update: {}", e);
              }
            }
            Err(e) => {
              tracing::error!("Emitter snapshot task panicked: {}; skipping tick", e);
            }
          }
        }
      });

      Ok(())
    })
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
      device_id: "cpu".to_string(),
    };

    let json = serde_json::to_string(&sensor).expect("Should serialize to JSON");
    assert!(json.contains("\"id\":\"cpu.temperature\""));
    assert!(json.contains("\"name\":\"CPU Temperature\""));
    assert!(json.contains("\"category\":\"temperature\""));
    assert!(json.contains("\"unit\":\"celsius\""));
    assert!(json.contains("\"device_id\":\"cpu\""));
  }

  #[test]
  fn test_empty_sensor_list() {
    let sensors: Vec<SensorInfo> = vec![];
    let json = serde_json::to_string(&sensors).expect("Should serialize empty list");
    assert_eq!(json, "[]");
  }

  #[test]
  fn test_sensor_reading_serialization() {
    let reading = SensorReading {
      id: "cpu.load".to_string(),
      name: "CPU Load".to_string(),
      category: "utilization".to_string(),
      unit: "percent".to_string(),
      value: Some(72.5),
      timestamp_ms: 1712345678000,
    };

    let json = serde_json::to_string(&reading).expect("Should serialize to JSON");
    assert!(json.contains("\"id\":\"cpu.load\""));
    assert!(json.contains("\"value\":72.5"));
    // Explicit rename overrides the camelCase default — confirm the IPC contract.
    assert!(json.contains("\"timestampMs\":1712345678000"));
    assert!(!json.contains("\"timestamp_ms\""));
  }

  #[test]
  fn test_sensor_reading_null_value_serialization() {
    let reading = SensorReading {
      id: "gpu.temp".to_string(),
      name: "GPU Temperature".to_string(),
      category: "temperature".to_string(),
      unit: "celsius".to_string(),
      value: None,
      timestamp_ms: 0,
    };

    let json = serde_json::to_string(&reading).expect("Should serialize to JSON");
    assert!(json.contains("\"value\":null"));
    assert!(json.contains("\"timestampMs\":0"));
  }

  #[tokio::test]
  async fn test_build_snapshot_with_polled_sensor() {
    use astragauge_domain::{SensorDescriptor, SensorId, SensorSample};

    let store = Arc::new(SensorStore::new());
    let sensor_id = SensorId::new("cpu.load").unwrap();

    store
      .register_sensor(SensorDescriptor {
        id: sensor_id.clone(),
        name: "CPU Load".to_string(),
        category: "utilization".to_string(),
        unit: "percent".to_string(),
        device: None,
        tags: vec![],
      })
      .await
      .unwrap();

    store
      .push_sample(SensorSample {
        sensor_id: sensor_id.clone(),
        timestamp_ms: 12345,
        value: Some(42.0),
      })
      .await
      .unwrap();

    let readings = build_snapshot(&store).await;

    assert_eq!(readings.len(), 1);
    let r = &readings[0];
    assert_eq!(r.id, "cpu.load");
    assert_eq!(r.value, Some(42.0));
    assert_eq!(r.timestamp_ms, 12345);
    assert_eq!(r.unit, "percent");
  }

  #[tokio::test]
  async fn test_build_snapshot_unpolled_sensor_returns_null() {
    use astragauge_domain::{SensorDescriptor, SensorId};

    let store = Arc::new(SensorStore::new());
    let sensor_id = SensorId::new("cpu.temperature").unwrap();

    store
      .register_sensor(SensorDescriptor {
        id: sensor_id.clone(),
        name: "CPU Temperature".to_string(),
        category: "temperature".to_string(),
        unit: "celsius".to_string(),
        device: None,
        tags: vec![],
      })
      .await
      .unwrap();

    // No push_sample — sensor registered but not yet polled.
    let readings = build_snapshot(&store).await;

    assert_eq!(readings.len(), 1);
    let r = &readings[0];
    assert_eq!(r.id, "cpu.temperature");
    assert_eq!(r.value, None);
    assert_eq!(r.timestamp_ms, 0);
  }
}
