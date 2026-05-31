use astragauge_domain::SensorId;
use astragauge_sensor_store::{SensorStore, StoreConfig};
use std::time::Duration;

mod common;

#[allow(unused_imports)]
use common::{make_descriptor, make_sample};

#[tokio::test]
async fn test_subscribe_receives_matching_sample() {
  let store = SensorStore::new();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  let mut sub = store.subscribe("cpu.*").await;

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(72.5)))
    .await
    .unwrap();

  let received = tokio::time::timeout(Duration::from_millis(100), sub.recv()).await;
  assert!(received.is_ok());
  let sample = received.unwrap().unwrap();
  assert_eq!(sample.sensor_id.as_str(), "cpu.temperature");
  assert_eq!(sample.value, Some(72.5));
}

#[tokio::test]
async fn test_subscribe_does_not_receive_non_matching() {
  let store = SensorStore::new();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("gpu.temperature"))
    .await
    .unwrap();

  let mut sub = store.subscribe("gpu.*").await;

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(72.5)))
    .await
    .unwrap();

  let received = tokio::time::timeout(Duration::from_millis(50), sub.recv()).await;
  assert!(received.is_err());
}

#[tokio::test]
async fn test_subscribe_wildcard_receives_multiple_sensors() {
  let store = SensorStore::new();
  store
    .register_sensor(make_descriptor("cpu.core0.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("cpu.core1.temperature"))
    .await
    .unwrap();

  let mut sub = store.subscribe("cpu.*.temperature").await;

  store
    .push_sample(make_sample("cpu.core0.temperature", 1000, Some(45.0)))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.core1.temperature", 1001, Some(46.0)))
    .await
    .unwrap();

  let r1 = tokio::time::timeout(Duration::from_millis(100), sub.recv()).await;
  let r2 = tokio::time::timeout(Duration::from_millis(100), sub.recv()).await;
  assert!(r1.is_ok());
  assert!(r2.is_ok());
}

#[tokio::test]
async fn test_push_samples_notifies_for_each_sample() {
  let store = SensorStore::new();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("memory.used"))
    .await
    .unwrap();

  let mut sub = store.subscribe("*.*").await;

  let samples = vec![
    make_sample("cpu.temperature", 1000, Some(72.5)),
    make_sample("memory.used", 1001, Some(8.0)),
  ];

  store.push_samples(&samples).await.unwrap();

  let r1 = tokio::time::timeout(Duration::from_millis(100), sub.recv()).await;
  let r2 = tokio::time::timeout(Duration::from_millis(100), sub.recv()).await;
  assert!(r1.is_ok());
  assert!(r2.is_ok());
}

#[tokio::test]
async fn test_unsubscribe_stops_notifications() {
  let store = SensorStore::new();
  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();

  let sub = store.subscribe("cpu.*").await;
  let sub_id = sub.id();
  drop(sub);

  store.unsubscribe(sub_id).await;

  let mut sub2 = store.subscribe("cpu.*").await;

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(72.5)))
    .await
    .unwrap();

  let received = tokio::time::timeout(Duration::from_millis(100), sub2.recv()).await;
  assert!(received.is_ok());

  let mut old_sub_check = store.subscribe("nonexistent.*").await;
  let received = tokio::time::timeout(Duration::from_millis(50), old_sub_check.recv()).await;
  assert!(received.is_err());
}

#[tokio::test]
async fn test_list_sensors_by_category() {
  let store = SensorStore::new();
  let mut d1 = make_descriptor("cpu.temperature");
  d1.category = "temperature".to_string();
  let mut d2 = make_descriptor("cpu.utilization");
  d2.category = "utilization".to_string();
  let mut d3 = make_descriptor("gpu.temperature");
  d3.category = "temperature".to_string();

  store.register_sensor(d1).await.unwrap();
  store.register_sensor(d2).await.unwrap();
  store.register_sensor(d3).await.unwrap();

  let temp_sensors = store.list_sensors_by_category("temperature").await;
  assert_eq!(temp_sensors.len(), 2);
  assert!(temp_sensors.contains(&SensorId::new("cpu.temperature").unwrap()));
  assert!(temp_sensors.contains(&SensorId::new("gpu.temperature").unwrap()));

  let util_sensors = store.list_sensors_by_category("utilization").await;
  assert_eq!(util_sensors.len(), 1);

  let empty = store.list_sensors_by_category("nonexistent").await;
  assert!(empty.is_empty());
}

#[tokio::test]
async fn test_query_pattern() {
  let store = SensorStore::new();
  store
    .register_sensor(make_descriptor("cpu.core0.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("cpu.core1.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("gpu.temperature"))
    .await
    .unwrap();

  let result = store.query_pattern("*.*.temperature").await;
  assert_eq!(result.len(), 2);

  let result = store.query_pattern("*.*").await;
  assert_eq!(result.len(), 1);

  let result = store.query_pattern("gpu.*").await;
  assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_get_stale_sensors() {
  let config = StoreConfig::new().with_staleness_threshold_ms(5000);
  let store = SensorStore::with_config(config);

  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();
  store
    .register_sensor(make_descriptor("gpu.temperature"))
    .await
    .unwrap();

  store
    .push_sample(make_sample("cpu.temperature", 1000, Some(50.0)))
    .await
    .unwrap();
  store
    .push_sample(make_sample("gpu.temperature", 10000, Some(60.0)))
    .await
    .unwrap();

  let stale = store.get_stale_sensors(8000).await;
  assert_eq!(stale.len(), 1);
  assert!(stale.contains(&SensorId::new("cpu.temperature").unwrap()));

  let stale = store.get_stale_sensors(1000).await;
  assert!(stale.is_empty());
}

#[tokio::test]
async fn test_sensor_count() {
  let store = SensorStore::new();
  assert_eq!(store.sensor_count().await, 0);

  store
    .register_sensor(make_descriptor("cpu.temperature"))
    .await
    .unwrap();
  assert_eq!(store.sensor_count().await, 1);

  store
    .register_sensor(make_descriptor("gpu.temperature"))
    .await
    .unwrap();
  assert_eq!(store.sensor_count().await, 2);

  store
    .unregister_sensor(&SensorId::new("cpu.temperature").unwrap())
    .await
    .unwrap();
  assert_eq!(store.sensor_count().await, 1);
}
