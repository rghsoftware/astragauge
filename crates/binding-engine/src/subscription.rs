use crate::engine::{parse_transform, BindingEngine};
use crate::types::{Binding, BindingResult, BindingSource, ResolvedBinding, Transform};
use astragauge_domain::SensorId;
use astragauge_sensor_store::pattern::match_pattern;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Default capacity for the broadcast notification channel.
///
/// This value balances memory usage against the ability to handle burst updates.
/// When the channel is full, slow receivers will miss notifications (lagging behavior).
pub const DEFAULT_NOTIFICATION_CHANNEL_CAPACITY: usize = 256;

/// Notification sent when a binding value changes.
#[derive(Debug, Clone)]
pub struct BindingNotification {
  pub binding_id: String,
  pub result: BindingResult<ResolvedBinding>,
}

/// Cached binding with pre-parsed transform for performance.
#[derive(Clone)]
struct CachedBinding {
  binding: Binding,
  parsed_transform: Option<Transform>,
}

/// Manages subscriptions for binding updates.
pub struct BindingSubscription {
  engine: BindingEngine,
  bindings: Arc<RwLock<HashMap<String, CachedBinding>>>,
  notifier: broadcast::Sender<BindingNotification>,
}

impl BindingSubscription {
  /// Creates a new `BindingSubscription` with the given engine.
  #[must_use]
  pub fn new(engine: BindingEngine) -> Self {
    let (notifier, _) = broadcast::channel(DEFAULT_NOTIFICATION_CHANNEL_CAPACITY);
    Self {
      engine,
      bindings: Arc::new(RwLock::new(HashMap::new())),
      notifier,
    }
  }

  /// Register a binding for updates.
  ///
  /// Returns an error if the transform string is invalid.
  pub async fn register(
    &self,
    binding_id: String,
    binding: Binding,
  ) -> crate::types::BindingResult<()> {
    let parsed_transform = match &binding.transform {
      Some(transform_str) => Some(parse_transform(transform_str)?),
      None => None,
    };

    let mut bindings = self.bindings.write().await;
    bindings.insert(
      binding_id,
      CachedBinding {
        binding,
        parsed_transform,
      },
    );
    Ok(())
  }

  /// Unregister a binding from updates.
  pub async fn unregister(&self, binding_id: &str) {
    let mut bindings = self.bindings.write().await;
    bindings.remove(binding_id);
  }

  /// Called when a sensor updates. Triggers recompute if binding depends on it.
  pub async fn on_sensor_update(&self, sensor_id: &SensorId) {
    let affected_ids: Vec<String> = {
      let bindings = self.bindings.read().await;
      bindings
        .iter()
        .filter(|(_, binding)| self.binding_uses_sensor(binding, sensor_id))
        .map(|(id, _)| id.clone())
        .collect()
    };

    for binding_id in affected_ids {
      let result = self.recompute(&binding_id).await;
      let _ = self.notifier.send(BindingNotification {
        binding_id: binding_id.clone(),
        result,
      });
    }
  }

  /// Subscribe to binding notifications.
  pub fn subscribe(&self) -> broadcast::Receiver<BindingNotification> {
    self.notifier.subscribe()
  }

  /// Manually trigger recompute for a binding.
  pub async fn recompute(&self, binding_id: &str) -> BindingResult<ResolvedBinding> {
    let cached = {
      let bindings = self.bindings.read().await;
      bindings.get(binding_id).cloned()
    };

    match cached {
      Some(cached) => {
        self
          .engine
          .resolve_with_transform(&cached.binding, cached.parsed_transform.as_ref())
          .await
      }
      None => Err(crate::types::BindingError::BindingNotFound(
        binding_id.to_string(),
      )),
    }
  }

  /// Returns the number of registered bindings.
  pub async fn binding_count(&self) -> usize {
    let bindings = self.bindings.read().await;
    bindings.len()
  }

  fn binding_uses_sensor(&self, cached: &CachedBinding, sensor_id: &SensorId) -> bool {
    match &cached.binding.source {
      BindingSource::Direct {
        sensor_id: direct_id,
      } => direct_id == sensor_id,
      BindingSource::Wildcard { pattern, .. } => {
        let matches = match_pattern(pattern, std::slice::from_ref(sensor_id));
        matches.contains(sensor_id)
      }
    }
  }
}

impl Clone for BindingSubscription {
  fn clone(&self) -> Self {
    Self {
      engine: self.engine.clone(),
      bindings: Arc::clone(&self.bindings),
      notifier: self.notifier.clone(),
    }
  }
}

impl std::fmt::Debug for BindingSubscription {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BindingSubscription")
      .field("engine", &"BindingEngine { .. }")
      .field("bindings", &"Arc<RwLock<HashMap>>")
      .field("notifier", &self.notifier)
      .finish_non_exhaustive()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::Aggregation;
  use astragauge_domain::{SensorDescriptor, SensorSample};
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

  #[tokio::test]
  async fn test_subscription_triggers_on_matching_sensor_update() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);
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

    subscription
      .on_sensor_update(&make_id("cpu.temperature"))
      .await;

    let notification = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
      .await
      .expect("should receive notification")
      .expect("notification should be ok");

    assert_eq!(notification.binding_id, "cpu_temp");
    assert_eq!(notification.result.unwrap().value, Some(42.5));
  }

  #[tokio::test]
  async fn test_subscription_ignores_non_matching_sensor_update() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);
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

    subscription
      .on_sensor_update(&make_id("gpu.temperature"))
      .await;

    let result = tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(
      result.is_err(),
      "should not receive notification for non-matching sensor"
    );
  }

  #[tokio::test]
  async fn test_multiple_bindings_share_notification_channel() {
    let store =
      make_store_with_sensors(&[("cpu.temperature", 40.0), ("gpu.temperature", 70.0)]).await;
    let engine = BindingEngine::new(store);
    let subscription = BindingSubscription::new(engine);

    let cpu_binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let gpu_binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("gpu.temperature"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    subscription
      .register("cpu_temp".to_string(), cpu_binding)
      .await
      .unwrap();
    subscription
      .register("gpu_temp".to_string(), gpu_binding)
      .await
      .unwrap();

    let mut receiver1 = subscription.subscribe();
    let mut receiver2 = subscription.subscribe();

    subscription
      .on_sensor_update(&make_id("cpu.temperature"))
      .await;

    let notif1 = tokio::time::timeout(Duration::from_millis(100), receiver1.recv())
      .await
      .expect("receiver1 should get notification")
      .expect("notification ok");

    let notif2 = tokio::time::timeout(Duration::from_millis(100), receiver2.recv())
      .await
      .expect("receiver2 should get notification")
      .expect("notification ok");

    assert_eq!(notif1.binding_id, "cpu_temp");
    assert_eq!(notif2.binding_id, "cpu_temp");
  }

  #[tokio::test]
  async fn test_lock_free_notification_completes_without_deadlock() {
    let store = make_store_with_sensors(&[
      ("cpu.core0.temperature", 40.0),
      ("cpu.core1.temperature", 50.0),
    ])
    .await;
    let engine = BindingEngine::new(store);
    let subscription = BindingSubscription::new(engine);

    for i in 0..10 {
      let binding = Binding {
        source: BindingSource::Wildcard {
          pattern: "cpu.core*.temperature".to_string(),
          aggregation: Aggregation::Avg,
        },
        transform: None,
        target_property: "value".to_string(),
      };
      subscription
        .register(format!("binding_{}", i), binding)
        .await
        .unwrap();
    }

    let mut receiver = subscription.subscribe();

    let result = tokio::time::timeout(
      Duration::from_millis(500),
      subscription.on_sensor_update(&make_id("cpu.core0.temperature")),
    )
    .await;

    assert!(
      result.is_ok(),
      "on_sensor_update should complete without deadlock"
    );

    let mut count = 0;
    while let Ok(Ok(_)) = tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await {
      count += 1;
      if count >= 10 {
        break;
      }
    }
    assert_eq!(
      count, 10,
      "should receive notifications for all 10 bindings"
    );
  }

  #[tokio::test]
  async fn test_wildcard_binding_triggered_by_matching_sensor() {
    let store = make_store_with_sensors(&[
      ("cpu.core0.temperature", 40.0),
      ("cpu.core1.temperature", 50.0),
    ])
    .await;
    let engine = BindingEngine::new(store);
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

    subscription
      .on_sensor_update(&make_id("cpu.core0.temperature"))
      .await;

    let notification = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
      .await
      .expect("should receive notification")
      .expect("notification ok");

    assert_eq!(notification.binding_id, "cpu_avg_temp");
    assert_eq!(notification.result.unwrap().value, Some(45.0));
  }

  #[tokio::test]
  async fn test_recompute_returns_binding_result() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);
    let subscription = BindingSubscription::new(engine);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: Some("round(0)".to_string()),
      target_property: "value".to_string(),
    };

    subscription
      .register("cpu_temp".to_string(), binding)
      .await
      .unwrap();

    let result = subscription.recompute("cpu_temp").await;

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert_eq!(resolved.value, Some(43.0));
    assert_eq!(resolved.source_count, 1);
  }

  #[tokio::test]
  async fn test_recompute_nonexistent_binding_returns_error() {
    let store = SensorStore::new();
    let engine = BindingEngine::new(store);
    let subscription = BindingSubscription::new(engine);

    let result = subscription.recompute("nonexistent").await;

    assert!(matches!(
      result,
      Err(crate::types::BindingError::BindingNotFound(_))
    ));
  }

  #[tokio::test]
  async fn test_unregister_removes_binding() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);
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

    assert_eq!(subscription.binding_count().await, 1);

    subscription.unregister("cpu_temp").await;

    assert_eq!(subscription.binding_count().await, 0);
  }

  #[tokio::test]
  async fn test_clone_shares_state() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);
    let subscription1 = BindingSubscription::new(engine);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    subscription1
      .register("cpu_temp".to_string(), binding)
      .await
      .unwrap();

    let subscription2 = subscription1.clone();
    assert_eq!(subscription2.binding_count().await, 1);

    subscription2.unregister("cpu_temp").await;
    assert_eq!(subscription1.binding_count().await, 0);
  }
}
