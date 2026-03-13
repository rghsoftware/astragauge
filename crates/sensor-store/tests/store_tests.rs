use astragauge_domain::SensorId;
use astragauge_sensor_store::{SensorStore, StoreConfig, StoreError};

mod common;

#[allow(unused_imports)]
use common::{make_descriptor, make_sample};

#[tokio::test]
async fn test_register_sensor_stores_descriptor() {
  let store = SensorStore::new();
  let descriptor = make_descriptor("cpu.temperature");

  store.register_sensor(descriptor.clone()).await.unwrap();

  let retrieved = store.get_descriptor(&descriptor.id).await;
  assert!(retrieved.is_some());
  let retrieved = retrieved.unwrap();
  assert_eq!(retrieved.id, descriptor.id);
  assert_eq!(retrieved.name, descriptor.name);
}

#[tokio::test]
async fn test_list_sensors_returns_all_registered() {
  let store = SensorStore::new();
  let id1 = SensorId::new("cpu.temperature").unwrap();
  let id2 = SensorId::new("gpu.utilization").unwrap();
  let id3 = SensorId::new("ram.used").unwrap();

  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("gpu.utilization"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("ram.used"))
    .await
    .unwrap();

  let sensors = store.list_sensors().await;
  assert_eq!(sensors.len(), 3);
  assert!(sensors.contains(&id1));
  assert!(sensors.contains(&id2));
  assert!(sensors.contains(&id3));
}

#[tokio::test]
async fn test_register_same_sensor_twice_succeeds() {
  let store = SensorStore::new();
  let descriptor = make_descriptor("cpu.temperature");

  let result1 = store.register_sensor(descriptor.clone()).await;
  let result2 = store.register_sensor(descriptor.clone()).await;

  assert!(result1.is_ok());
  assert!(result2.is_ok());

  let sensors = store.list_sensors().await;
  assert_eq!(sensors.len(), 1);
}

#[tokio::test]
async fn test_get_descriptor_for_unknown_returns_none() {
  let store = SensorStore::new();
  let unknown_id = SensorId::new("cpu.temperature").unwrap();

  let result = store.get_descriptor(&unknown_id).await;

  assert!(result.is_none());
}

#[tokio::test]
async fn test_push_sample_updates_value() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  let sample = make_sample("cpu.temperature", 1000, Some(72.5));
  store.push_sample(sample.clone()).await.unwrap();

  let value = store.get_value(&id).await;
  assert!(value.is_some());
  let value = value.unwrap();
  assert_eq!(value.sensor_id, id);
  assert_eq!(value.value, Some(72.5));
}

#[tokio::test]
async fn test_push_sample_overwrites_previous() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(72.5)))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.temperature", 2000, Some(75.0)))
    .await
    .unwrap();

  let value = store.get_value(&id).await.unwrap();
  assert_eq!(value.value, Some(75.0));
  assert_eq!(value.timestamp_ms, 2000);
}

#[tokio::test]
async fn test_push_sample_for_unregistered_returns_error() {
  let store = SensorStore::new();
  let sample = make_sample("cpu.temperature", 1000, Some(72.5));

  let result = store.push_sample(sample).await;

  assert!(result.is_err());
  match result {
    Err(StoreError::UnknownSensor { id }) => {
      assert_eq!(id, SensorId::new("cpu.temperature").unwrap());
    }
    _ => panic!("Expected UnknownSensor error"),
  }
}

#[tokio::test]
async fn test_push_null_value_stored() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  let sample = make_sample("cpu.temperature", 1000, None);
  store.push_sample(sample).await.unwrap();

  let value = store.get_value(&id).await.unwrap();
  assert_eq!(value.value, None);
}

#[tokio::test]
async fn test_fresh_sensor_not_stale() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 10000, Some(72.5)))
    .await
    .unwrap();

  let is_stale = store.is_stale(&id, 12000).await;
  assert!(!is_stale);
}

#[tokio::test]
async fn test_old_sample_is_stale() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(72.5)))
    .await
    .unwrap();

  let is_stale = store.is_stale(&id, 10000).await;
  assert!(is_stale);
}

#[tokio::test]
async fn test_staleness_threshold_configurable() {
  let config = StoreConfig::new()
    .with_history_capacity(120)
    .with_staleness_threshold_ms(10000);
  let store = SensorStore::with_config(config);
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(72.5)))
    .await
    .unwrap();

  let is_stale = store.is_stale(&id, 9000).await;
  assert!(!is_stale);

  let is_stale = store.is_stale(&id, 12000).await;
  assert!(is_stale);
}

#[tokio::test]
async fn test_unregistered_sensor_is_stale() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();

  let is_stale = store.is_stale(&id, 1000).await;
  assert!(is_stale);
}

#[tokio::test]
async fn test_push_sample_adds_to_history() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(70.0)))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.temperature", 2000, Some(71.0)))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.temperature", 3000, Some(72.0)))
    .await
    .unwrap();

  let history = store.get_history(&id).await.unwrap();
  assert_eq!(history.len(), 3);
  assert_eq!(history[0].value, Some(70.0));
  assert_eq!(history[1].value, Some(71.0));
  assert_eq!(history[2].value, Some(72.0));
}

#[tokio::test]
async fn test_history_respects_capacity() {
  let config = StoreConfig::new()
    .with_history_capacity(5)
    .with_staleness_threshold_ms(5000);
  let store = SensorStore::with_config(config);
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  for i in 0..10 {
    store
      .push_sample(make_sample("cpu.temperature", i * 1000, Some(i as f64)))
      .await
      .unwrap();
  }

  let history = store.get_history(&id).await.unwrap();
  assert_eq!(history.len(), 5);
  assert_eq!(history[0].value, Some(5.0));
  assert_eq!(history[4].value, Some(9.0));
}

#[tokio::test]
async fn test_history_for_unregistered_returns_none() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();

  let history = store.get_history(&id).await;
  assert!(history.is_none());
}

#[tokio::test]
async fn test_get_value_for_unknown_returns_none() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();

  let value = store.get_value(&id).await;
  assert!(value.is_none());
}

#[tokio::test]
async fn test_get_value_with_timestamp() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 12345, Some(72.5)))
    .await
    .unwrap();

  let result = store.get_value_with_timestamp(&id).await;
  assert!(result.is_some());
  let (sample, timestamp) = result.unwrap();
  assert_eq!(sample.value, Some(72.5));
  assert_eq!(timestamp, 12345);
}

#[tokio::test]
async fn test_get_value_with_timestamp_for_unknown_returns_none() {
  let store = SensorStore::new();
  let id = SensorId::new("cpu.temperature").unwrap();

  let result = store.get_value_with_timestamp(&id).await;
  assert!(result.is_none());
}
