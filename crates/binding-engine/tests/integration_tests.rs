use astragauge_binding_engine::{
  engine::BindingEngine, subscription::BindingSubscription, Aggregation, Binding, BindingError,
  BindingSource,
};
use astragauge_domain::{SensorDescriptor, SensorId, SensorSample};
use astragauge_sensor_store::SensorStore;
use std::time::Duration;

fn make_id(s: &str) -> SensorId {
  SensorId::new(s).unwrap()
}

async fn make_store_with_sensors(sensors: &[(&str, f64)]) -> SensorStore {
  let store = SensorStore::new();
  for (id, value) in sensors {
    let sensor_id = make_id(id);
    let descriptor = SensorDescriptor {
      id: sensor_id.clone(),
      name: format!("Sensor {}", id),
      unit: "unit".to_string(),
      category: "default".to_string(),
      device: None,
      tags: vec![],
    };
    store.register_sensor(descriptor).await.unwrap();

    let sample = SensorSample {
      sensor_id,
      value: Some(*value),
      timestamp_ms: 1000,
    };
    store.push_sample(sample).await.unwrap();
  }
  store
}

async fn update_sensor_value(store: &SensorStore, id: &str, value: f64, timestamp_ms: u64) {
  let sample = SensorSample {
    sensor_id: make_id(id),
    value: Some(value),
    timestamp_ms,
  };
  store.push_sample(sample).await.unwrap();
}

#[tokio::test]
async fn integration_direct_binding_pipeline() {
  let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
  let engine = BindingEngine::new(store.clone());

  let binding = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("cpu.temperature"),
    },
    transform: None,
    target_property: "value".to_string(),
  };

  let result = engine.resolve(&binding).await.unwrap();
  assert_eq!(result.value, Some(42.5));
  assert_eq!(result.source_count, 1);

  update_sensor_value(&store, "cpu.temperature", 55.0, 2000).await;

  let result = engine.resolve(&binding).await.unwrap();
  assert_eq!(result.value, Some(55.0));
  assert_eq!(result.source_count, 1);
}

#[tokio::test]
async fn integration_wildcard_binding_pipeline() {
  let store = make_store_with_sensors(&[
    ("cpu.core0.temperature", 40.0),
    ("cpu.core1.temperature", 50.0),
    ("cpu.core2.temperature", 60.0),
    ("gpu.temperature", 70.0),
  ])
  .await;
  let engine = BindingEngine::new(store.clone());

  let binding = Binding {
    source: BindingSource::Wildcard {
      pattern: "cpu.core*.temperature".to_string(),
      aggregation: Aggregation::Avg,
    },
    transform: None,
    target_property: "value".to_string(),
  };

  let result = engine.resolve(&binding).await.unwrap();
  assert_eq!(result.value, Some(50.0));
  assert_eq!(result.source_count, 3);

  update_sensor_value(&store, "cpu.core0.temperature", 70.0, 2000).await;

  let result = engine.resolve(&binding).await.unwrap();
  assert_eq!(result.value, Some(60.0));
  assert_eq!(result.source_count, 3);
}

#[tokio::test]
async fn integration_error_nonexistent_sensor() {
  let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
  let engine = BindingEngine::new(store);

  let binding = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("gpu.temperature"),
    },
    transform: None,
    target_property: "value".to_string(),
  };

  let result = engine.resolve(&binding).await;
  assert!(matches!(result, Err(BindingError::UnresolvedSensor(_))));

  if let Err(BindingError::UnresolvedSensor(id)) = result {
    assert_eq!(id, "gpu.temperature");
  }
}

#[tokio::test]
async fn integration_error_wildcard_no_match() {
  let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
  let engine = BindingEngine::new(store);

  let binding = Binding {
    source: BindingSource::Wildcard {
      pattern: "gpu.*.temperature".to_string(),
      aggregation: Aggregation::Avg,
    },
    transform: None,
    target_property: "value".to_string(),
  };

  let result = engine.resolve(&binding).await;
  assert!(matches!(result, Err(BindingError::WildcardNoMatch(_))));

  if let Err(BindingError::WildcardNoMatch(pattern)) = result {
    assert_eq!(pattern, "gpu.*.temperature");
  }
}

#[tokio::test]
async fn integration_subscription_receives_updates() {
  let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
  let engine = BindingEngine::new(store.clone());
  let subscription = BindingSubscription::new(engine);

  let binding = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("cpu.temperature"),
    },
    transform: None,
    target_property: "value".to_string(),
  };

  subscription
    .register("cpu_temp".to_string(), binding)
    .await
    .unwrap();

  let mut receiver = subscription.subscribe();

  update_sensor_value(&store, "cpu.temperature", 55.0, 2000).await;
  subscription
    .on_sensor_update(&make_id("cpu.temperature"))
    .await;

  let notification = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
    .await
    .expect("should receive notification")
    .expect("notification should be ok");

  assert_eq!(notification.binding_id, "cpu_temp");
  let resolved = notification.result.expect("should have resolved value");
  assert_eq!(resolved.value, Some(55.0));
  assert_eq!(resolved.source_count, 1);
}

#[tokio::test]
async fn integration_full_pipeline_with_transform() {
  let store = make_store_with_sensors(&[("cpu.utilization", 0.753)]).await;
  let engine = BindingEngine::new(store);

  let binding = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("cpu.utilization"),
    },
    transform: Some("percent".to_string()),
    target_property: "value".to_string(),
  };

  let result = engine.resolve(&binding).await.unwrap();
  assert_eq!(result.value, Some(75.3));
  assert_eq!(result.source_count, 1);
}

#[tokio::test]
async fn integration_wildcard_with_all_aggregations() {
  let store = make_store_with_sensors(&[
    ("cpu.core0.load", 10.0),
    ("cpu.core1.load", 20.0),
    ("cpu.core2.load", 30.0),
  ])
  .await;
  let engine = BindingEngine::new(store);

  let test_cases = [
    (Aggregation::Avg, Some(20.0)),
    (Aggregation::Min, Some(10.0)),
    (Aggregation::Max, Some(30.0)),
    (Aggregation::Sum, Some(60.0)),
    (Aggregation::Count, Some(3.0)),
  ];

  for (aggregation, expected) in test_cases {
    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "cpu.core*.load".to_string(),
        aggregation,
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, expected, "failed for {:?}", aggregation);
    assert_eq!(result.source_count, 3);
  }
}

#[tokio::test]
async fn integration_subscription_wildcard_triggered_by_matching_sensor() {
  let store = make_store_with_sensors(&[
    ("cpu.core0.temperature", 40.0),
    ("cpu.core1.temperature", 50.0),
  ])
  .await;
  let engine = BindingEngine::new(store.clone());
  let subscription = BindingSubscription::new(engine);

  let binding = Binding {
    source: BindingSource::Wildcard {
      pattern: "cpu.core*.temperature".to_string(),
      aggregation: Aggregation::Avg,
    },
    transform: None,
    target_property: "value".to_string(),
  };

  subscription
    .register("cpu_avg_temp".to_string(), binding)
    .await
    .unwrap();

  let mut receiver = subscription.subscribe();

  update_sensor_value(&store, "cpu.core1.temperature", 70.0, 2000).await;
  subscription
    .on_sensor_update(&make_id("cpu.core1.temperature"))
    .await;

  let notification = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
    .await
    .expect("should receive notification")
    .expect("notification ok");

  assert_eq!(notification.binding_id, "cpu_avg_temp");
  let resolved = notification.result.expect("should have resolved value");
  assert_eq!(resolved.value, Some(55.0));
}

#[tokio::test]
async fn integration_null_value_propagation() {
  let store = SensorStore::new();
  let sensor_id = make_id("cpu.temperature");
  let descriptor = SensorDescriptor {
    id: sensor_id.clone(),
    name: "CPU Temperature".to_string(),
    unit: "°C".to_string(),
    category: "default".to_string(),
    device: None,
    tags: vec![],
  };
  store.register_sensor(descriptor).await.unwrap();

  let null_sample = SensorSample {
    sensor_id: make_id("cpu.temperature"),
    value: None,
    timestamp_ms: 1000,
  };
  store.push_sample(null_sample).await.unwrap();

  let engine = BindingEngine::new(store);

  let binding = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("cpu.temperature"),
    },
    transform: Some("round(1)".to_string()),
    target_property: "value".to_string(),
  };

  let result = engine.resolve(&binding).await.unwrap();
  assert_eq!(result.value, None);
  assert_eq!(result.source_count, 1);
}

#[tokio::test]
async fn integration_multiple_subscriptions_share_engine() {
  let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
  let engine = BindingEngine::new(store.clone());
  let subscription = BindingSubscription::new(engine);

  let binding1 = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("cpu.temperature"),
    },
    transform: None,
    target_property: "value".to_string(),
  };

  let binding2 = Binding {
    source: BindingSource::Direct {
      sensor_id: make_id("cpu.temperature"),
    },
    transform: Some("round(0)".to_string()),
    target_property: "rounded_value".to_string(),
  };

  subscription
    .register("temp1".to_string(), binding1)
    .await
    .unwrap();
  subscription
    .register("temp2".to_string(), binding2)
    .await
    .unwrap();

  let mut receiver = subscription.subscribe();

  update_sensor_value(&store, "cpu.temperature", 55.7, 2000).await;
  subscription
    .on_sensor_update(&make_id("cpu.temperature"))
    .await;

  let mut received_binding_ids: Vec<String> = Vec::new();
  while let Ok(Ok(notif)) = tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await {
    received_binding_ids.push(notif.binding_id);
    if received_binding_ids.len() >= 2 {
      break;
    }
  }

  assert_eq!(received_binding_ids.len(), 2);
}
