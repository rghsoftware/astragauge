use crate::pattern::matches_single;
use astragauge_domain::{SensorId, SensorSample};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;

static NEXT_SUBSCRIPTION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

/// Manages broadcast subscriptions for sensor samples
pub struct SubscriptionManager {
  subscribers: HashMap<String, broadcast::Sender<SensorSample>>,
  subscription_patterns: HashMap<SubscriptionId, String>,
  default_capacity: usize,
}

/// A subscription to receive sensor samples matching a pattern
pub struct Subscription {
  id: SubscriptionId,
  receiver: broadcast::Receiver<SensorSample>,
  pattern: String,
}

impl SubscriptionManager {
  #[must_use]
  pub fn new() -> Self {
    Self::with_capacity(256)
  }

  #[must_use]
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      subscribers: HashMap::new(),
      subscription_patterns: HashMap::new(),
      default_capacity: capacity,
    }
  }

  pub fn subscribe(&mut self, pattern: &str) -> Subscription {
    let id = SubscriptionId(NEXT_SUBSCRIPTION_ID.fetch_add(1, Ordering::Relaxed));
    let sender = self
      .subscribers
      .entry(pattern.to_string())
      .or_insert_with(|| broadcast::channel(self.default_capacity).0);

    self.subscription_patterns.insert(id, pattern.to_string());

    Subscription {
      id,
      receiver: sender.subscribe(),
      pattern: pattern.to_string(),
    }
  }

  pub fn unsubscribe(&mut self, id: SubscriptionId) {
    if let Some(pattern) = self.subscription_patterns.remove(&id) {
      if let Some(sender) = self.subscribers.get(&pattern) {
        if sender.receiver_count() == 0 {
          self.subscribers.remove(&pattern);
        }
      }
    }
  }

  pub fn cleanup_empty_patterns(&mut self) {
    self
      .subscribers
      .retain(|_, sender| sender.receiver_count() > 0);
  }

  pub fn notify(&self, sample: &SensorSample, matches: impl Fn(&str, &SensorId) -> bool) {
    for (pattern, sender) in &self.subscribers {
      if matches(pattern, &sample.sensor_id) {
        let _ = sender.send(sample.clone());
      }
    }
  }

  pub fn notify_matching(&self, sample: &SensorSample) {
    for (pattern, sender) in &self.subscribers {
      if matches_single(pattern, &sample.sensor_id) {
        let _ = sender.send(sample.clone());
      }
    }
  }

  pub fn subscription_count(&self) -> usize {
    self.subscription_patterns.len()
  }

  pub fn pattern_count(&self) -> usize {
    self.subscribers.len()
  }
}

impl Subscription {
  pub fn id(&self) -> SubscriptionId {
    self.id
  }

  pub fn pattern(&self) -> &str {
    &self.pattern
  }

  pub async fn recv(&mut self) -> Result<SensorSample, broadcast::error::RecvError> {
    self.receiver.recv().await
  }
}

impl Default for SubscriptionManager {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_subscribe_creates_subscription() {
    let mut manager = SubscriptionManager::new();
    let _sub = manager.subscribe("cpu.*");
    assert!(manager.subscribers.contains_key("cpu.*"));
  }

  #[test]
  fn test_with_capacity() {
    let manager = SubscriptionManager::with_capacity(128);
    assert_eq!(manager.default_capacity, 128);
  }

  #[test]
  fn test_default() {
    let manager = SubscriptionManager::default();
    assert_eq!(manager.default_capacity, 256);
  }

  #[test]
  fn test_subscribe_same_pattern_reuses_channel() {
    let mut manager = SubscriptionManager::new();
    let _sub1 = manager.subscribe("cpu.*");
    let sub2 = manager.subscribe("cpu.*");

    assert_eq!(sub2.pattern(), "cpu.*");
    assert_eq!(manager.subscribers.len(), 1);
  }

  #[test]
  fn test_subscription_has_unique_id() {
    let mut manager = SubscriptionManager::new();
    let sub1 = manager.subscribe("cpu.*");
    let sub2 = manager.subscribe("cpu.*");
    let sub3 = manager.subscribe("gpu.*");

    assert_ne!(sub1.id(), sub2.id());
    assert_ne!(sub2.id(), sub3.id());
  }

  #[test]
  fn test_unsubscribe_removes_tracking() {
    let mut manager = SubscriptionManager::new();
    let sub = manager.subscribe("cpu.*");
    let id = sub.id();

    assert_eq!(manager.subscription_count(), 1);
    drop(sub);
    manager.unsubscribe(id);
    assert_eq!(manager.subscription_count(), 0);
  }

  #[test]
  fn test_cleanup_empty_patterns() {
    let mut manager = SubscriptionManager::new();
    {
      let _sub = manager.subscribe("cpu.*");
      assert_eq!(manager.pattern_count(), 1);
    }
    manager.cleanup_empty_patterns();
    assert_eq!(manager.pattern_count(), 0);
  }

  #[tokio::test]
  async fn test_notify_sends_to_matching_subscriptions() {
    let mut manager = SubscriptionManager::new();
    let mut sub1 = manager.subscribe("cpu.*");
    let mut sub2 = manager.subscribe("gpu.*");

    let sample = SensorSample {
      sensor_id: SensorId::new("cpu.temperature").unwrap(),
      timestamp_ms: 1712345678,
      value: Some(45.0),
    };

    manager.notify(&sample, |pattern, sensor_id| {
      pattern == "cpu.*" && sensor_id.as_str().starts_with("cpu.")
    });

    let received = tokio::time::timeout(std::time::Duration::from_millis(100), sub1.recv()).await;
    assert!(received.is_ok());

    let received = tokio::time::timeout(std::time::Duration::from_millis(100), sub2.recv()).await;
    assert!(received.is_err());
  }

  #[tokio::test]
  async fn test_notify_matching_uses_pattern_matching() {
    let mut manager = SubscriptionManager::new();
    let mut sub1 = manager.subscribe("cpu.*.*");
    let mut sub2 = manager.subscribe("gpu.*.*");
    let mut sub3 = manager.subscribe("cpu.core*.temperature");

    let sample = SensorSample {
      sensor_id: SensorId::new("cpu.core0.temperature").unwrap(),
      timestamp_ms: 1712345678,
      value: Some(45.0),
    };

    manager.notify_matching(&sample);

    assert!(
      tokio::time::timeout(std::time::Duration::from_millis(100), sub1.recv())
        .await
        .is_ok()
    );
    assert!(
      tokio::time::timeout(std::time::Duration::from_millis(100), sub3.recv())
        .await
        .is_ok()
    );
    assert!(
      tokio::time::timeout(std::time::Duration::from_millis(100), sub2.recv())
        .await
        .is_err()
    );
  }
}
