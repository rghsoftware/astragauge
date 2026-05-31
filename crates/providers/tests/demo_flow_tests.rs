//! End-to-end data-flow test for the demo MockProvider.
//!
//! Proves the headless half of the vertical slice's pipeline:
//! Provider -> ProviderHost (discover + poll) -> SensorStore -> snapshot read.
//! This exercises the same path the Tauri `get_sensor_snapshot` command and the
//! background `sensors-update` emitter rely on, without requiring a GUI.

use std::sync::Arc;
use std::time::Duration;

use astragauge_domain::SensorId;
use astragauge_provider_host::{HostConfig, Provider, ProviderHost};
use astragauge_providers::MockProvider;
use astragauge_sensor_store::SensorStore;

/// The sensors the demo provider is contracted to expose, paired with a
/// generous plausible upper bound (percentages/temps stay <= 100; clock in MHz
/// and memory in MB range much higher).
const EXPECTED_SENSORS: [(&str, f64); 7] = [
  ("cpu.total.utilization", 100.0),
  ("cpu.clock", 5200.0),
  ("cpu.temperature", 100.0),
  ("gpu.total.utilization", 100.0),
  ("gpu.temperature", 100.0),
  ("memory.used.percent", 100.0),
  ("memory.used", 32768.0),
];

#[tokio::test]
async fn demo_provider_flows_live_values_into_store() {
  let store = Arc::new(SensorStore::new());
  let mut host = ProviderHost::new(HostConfig::default(), Arc::clone(&store));

  let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(MockProvider::new_demo()));
  host
    .register_provider(provider)
    .expect("demo provider should register");

  host.start();

  // The demo polls every 500ms; wait for a few cycles so descriptors are
  // registered and at least one batch of samples has landed in the store.
  tokio::time::sleep(Duration::from_millis(1500)).await;

  let sensor_ids = store.list_sensors().await;
  assert_eq!(
    sensor_ids.len(),
    EXPECTED_SENSORS.len(),
    "expected {} sensors registered, found {}",
    EXPECTED_SENSORS.len(),
    sensor_ids.len()
  );

  for (id_str, ceiling) in EXPECTED_SENSORS {
    let id = SensorId::new(id_str).expect("valid sensor id");

    let descriptor = store
      .get_descriptor(&id)
      .await
      .unwrap_or_else(|| panic!("descriptor for {id_str} should be registered"));
    assert_eq!(descriptor.id.as_str(), id_str);

    let sample = store
      .get_value(&id)
      .await
      .unwrap_or_else(|| panic!("a sample for {id_str} should have been polled"));
    let value = sample
      .value
      .unwrap_or_else(|| panic!("{id_str} should have a concrete value, not null"));

    assert!(
      (0.0..=ceiling).contains(&value),
      "{id_str} value {value} should be within its contracted 0..={ceiling} range"
    );
  }

  let _ = host.shutdown().await;
}

#[tokio::test]
async fn demo_provider_values_change_over_time() {
  // Proves values are time-varying (the slice's reason for existing: the UI
  // should visibly update), reading directly from the provider so the test is
  // not coupled to polling cadence.
  let provider = MockProvider::new_demo();
  let id = SensorId::new("cpu.total.utilization").expect("valid sensor id");

  let read = |samples: &[astragauge_domain::SensorSample]| -> f64 {
    samples
      .iter()
      .find(|s| s.sensor_id == id)
      .and_then(|s| s.value)
      .expect("cpu.total.utilization should be present with a value")
  };

  let first = read(&provider.poll().await.expect("poll succeeds"));
  tokio::time::sleep(Duration::from_millis(60)).await;
  let second = read(&provider.poll().await.expect("poll succeeds"));

  assert!(
    (first - second).abs() > f64::EPSILON,
    "demo values should change between polls (got {first} then {second})"
  );
}
