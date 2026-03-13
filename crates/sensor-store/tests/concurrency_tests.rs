use astragauge_domain::SensorId;
use astragauge_sensor_store::{SensorStore, SubscriptionManager};
use std::sync::Arc;
use tokio::sync::Mutex;

mod common;

#[allow(unused_imports)]
use common::{make_descriptor, make_sample};

#[tokio::test]
async fn test_concurrent_reads_dont_block() {
  let store = Arc::new(SensorStore::new());
  store
    .register_sensor(make_descriptor("cpu.temp"))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.temp", 1000, Some(50.0)))
    .await
    .unwrap();

  let mut handles = vec![];
  for _ in 0..10 {
    let s = Arc::clone(&store);
    handles.push(tokio::spawn(async move {
      s.get_value(&SensorId::new("cpu.temp").unwrap()).await
    }));
  }

  for h in handles {
    let result = h.await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().value, Some(50.0));
  }
}

#[tokio::test]
async fn test_write_during_read_doesnt_deadlock() {
  let store = Arc::new(SensorStore::new());
  store
    .register_sensor(make_descriptor("cpu.temp"))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.temp", 1000, Some(50.0)))
    .await
    .unwrap();

  let barrier = Arc::new(tokio::sync::Barrier::new(2));

  let store_reader = Arc::clone(&store);
  let barrier_reader = Arc::clone(&barrier);
  let reader = tokio::spawn(async move {
    let _value = store_reader
      .get_value(&SensorId::new("cpu.temp").unwrap())
      .await;
    barrier_reader.wait().await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
  });

  let store_writer = Arc::clone(&store);
  let barrier_writer = Arc::clone(&barrier);
  let writer = tokio::spawn(async move {
    barrier_writer.wait().await;
    store_writer
      .push_sample(make_sample("cpu.temp", 2000, Some(60.0)))
      .await
      .unwrap();
  });

  let result = tokio::time::timeout(std::time::Duration::from_millis(500), async {
    reader.await.unwrap();
    writer.await.unwrap();
  })
  .await;

  assert!(result.is_ok(), "Deadlock detected: operations timed out");

  let final_value = store.get_value(&SensorId::new("cpu.temp").unwrap()).await;
  assert_eq!(final_value.unwrap().value, Some(60.0));
}

#[tokio::test]
async fn test_concurrent_subscriptions() {
  let manager = Arc::new(Mutex::new(SubscriptionManager::new()));
  let sample = make_sample("cpu.temperature", 1000, Some(45.0));

  let mut subscriptions = vec![];
  for _ in 0..5 {
    let m = Arc::clone(&manager);
    let sub = tokio::spawn(async move { m.lock().await.subscribe("cpu.*") }).await;
    subscriptions.push(sub.unwrap());
  }

  manager.lock().await.notify(&sample, |pattern, sensor_id| {
    pattern == "cpu.*" && sensor_id.as_str().starts_with("cpu.")
  });

  for mut sub in subscriptions {
    let received = tokio::time::timeout(std::time::Duration::from_millis(100), sub.recv()).await;
    assert!(
      received.is_ok(),
      "Subscription should have received the sample"
    );
  }
}

#[tokio::test]
async fn test_concurrent_writers_dont_corrupt_state() {
  let store = Arc::new(SensorStore::new());
  store
    .register_sensor(make_descriptor("cpu.temp"))
    .await
    .unwrap();

  let mut handles = vec![];
  for i in 0..10 {
    let s = Arc::clone(&store);
    handles.push(tokio::spawn(async move {
      s.push_sample(make_sample("cpu.temp", i * 1000, Some(i as f64)))
        .await
        .unwrap();
    }));
  }

  for h in handles {
    h.await.unwrap();
  }

  let final_value = store.get_value(&SensorId::new("cpu.temp").unwrap()).await;
  assert!(final_value.is_some());

  let history = store.get_history(&SensorId::new("cpu.temp").unwrap()).await;
  assert!(history.is_some());
  assert_eq!(history.unwrap().len(), 10);
}

#[tokio::test]
async fn test_mixed_concurrent_operations() {
  let store = Arc::new(SensorStore::new());
  store
    .register_sensor(make_descriptor("cpu.temp"))
    .await
    .unwrap();
  store
    .push_sample(make_sample("cpu.temp", 1000, Some(50.0)))
    .await
    .unwrap();

  let mut reader_handles = vec![];
  let mut writer_handles = vec![];

  for _ in 0..5 {
    let s = Arc::clone(&store);
    reader_handles.push(tokio::spawn(async move {
      s.get_value(&SensorId::new("cpu.temp").unwrap()).await
    }));
  }

  for i in 0..5 {
    let s = Arc::clone(&store);
    writer_handles.push(tokio::spawn(async move {
      s.push_sample(make_sample("cpu.temp", (i + 2) * 1000, Some(i as f64)))
        .await
        .unwrap();
    }));
  }

  let result = tokio::time::timeout(std::time::Duration::from_millis(500), async {
    for h in reader_handles {
      let _ = h.await;
    }
    for h in writer_handles {
      let _ = h.await;
    }
  })
  .await;

  assert!(result.is_ok(), "Mixed concurrent operations timed out");
}
