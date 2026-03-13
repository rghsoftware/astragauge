use crate::pattern::match_pattern;
use astragauge_domain::{SensorId, SensorSample};
use std::collections::HashMap;
use tokio::sync::broadcast;

/// Manages broadcast subscriptions for sensor samples
pub struct SubscriptionManager {
  subscribers: HashMap<String, broadcast::Sender<SensorSample>>,
  default_capacity: usize,
}

/// A subscription to receive sensor samples matching a pattern
pub struct Subscription {
  receiver: broadcast::Receiver<SensorSample>,
  pub pattern: String,
}

impl SubscriptionManager {
  /// Create a new SubscriptionManager with default channel capacity (256)
  pub fn new() -> Self {
    Self::with_capacity(256)
  }

  /// Create a new SubscriptionManager with specified channel capacity
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      subscribers: HashMap::new(),
      default_capacity: capacity,
    }
  }

  /// Subscribe to samples matching the given pattern
  pub fn subscribe(&mut self, pattern: &str) -> Subscription {
    let sender = self
      .subscribers
      .entry(pattern.to_string())
      .or_insert_with(|| broadcast::channel(self.default_capacity).0);

    Subscription {
      receiver: sender.subscribe(),
      pattern: pattern.to_string(),
    }
  }

  /// Notify all matching subscriptions of a new sample
  pub fn notify(&self, sample: &SensorSample, matches: impl Fn(&str, &SensorId) -> bool) {
    for (pattern, sender) in &self.subscribers {
      if matches(pattern, &sample.sensor_id) {
        // Ignore lagged errors - slow subscribers will miss messages
        let _ = sender.send(sample.clone());
      }
    }
  }

  /// Notify subscriptions matching the sample's sensor ID using pattern matching.
  ///
  /// This is a convenience method that uses [`match_pattern`] internally.
  /// For custom matching logic, use [`notify`](Self::notify) with a custom closure.
  pub fn notify_matching(&self, sample: &SensorSample) {
    for (pattern, sender) in &self.subscribers {
      let matching = match_pattern(pattern, std::slice::from_ref(&sample.sensor_id));
      if !matching.is_empty() {
        let _ = sender.send(sample.clone());
      }
    }
  }
}

impl Subscription {
  /// Receive the next sample from this subscription
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

    assert_eq!(sub2.pattern, "cpu.*");
    assert_eq!(manager.subscribers.len(), 1);
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
