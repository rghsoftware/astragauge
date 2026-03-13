use astragauge_domain::{SensorDescriptor, SensorId, SensorSample};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{RingBuffer, StoreError, StoreResult};

#[derive(Debug, Clone)]
pub struct StoreConfig {
  pub history_capacity: usize,
  pub staleness_threshold_ms: u64,
}

impl Default for StoreConfig {
  fn default() -> Self {
    Self {
      history_capacity: 120,
      staleness_threshold_ms: 5000,
    }
  }
}

struct SensorStoreInner {
  descriptors: HashMap<SensorId, SensorDescriptor>,
  values: HashMap<SensorId, SensorSample>,
  last_update: HashMap<SensorId, u64>,
  history: HashMap<SensorId, RingBuffer<SensorSample>>,
  config: StoreConfig,
}

#[derive(Clone)]
pub struct SensorStore {
  inner: Arc<RwLock<SensorStoreInner>>,
}

impl SensorStore {
  pub fn new() -> Self {
    Self::with_config(StoreConfig::default())
  }

  pub fn with_config(config: StoreConfig) -> Self {
    Self {
      inner: Arc::new(RwLock::new(SensorStoreInner {
        descriptors: HashMap::new(),
        values: HashMap::new(),
        last_update: HashMap::new(),
        history: HashMap::new(),
        config,
      })),
    }
  }

  pub async fn register_sensor(&self, descriptor: SensorDescriptor) -> StoreResult<()> {
    let mut store = self.inner.write().await;
    let id = descriptor.id.clone();
    let history_capacity = store.config.history_capacity;
    store.descriptors.insert(id.clone(), descriptor);
    store.history.insert(id, RingBuffer::new(history_capacity));
    Ok(())
  }

  pub async fn get_descriptor(&self, id: &SensorId) -> Option<SensorDescriptor> {
    let store = self.inner.read().await;
    store.descriptors.get(id).cloned()
  }

  pub async fn list_sensors(&self) -> Vec<SensorId> {
    let store = self.inner.read().await;
    store.descriptors.keys().cloned().collect()
  }

  pub async fn push_sample(&self, sample: SensorSample) -> StoreResult<()> {
    let mut store = self.inner.write().await;

    if !store.descriptors.contains_key(&sample.sensor_id) {
      return Err(StoreError::UnknownSensor {
        id: sample.sensor_id.clone(),
      });
    }

    let id = sample.sensor_id.clone();
    let timestamp = sample.timestamp_ms;

    store.values.insert(id.clone(), sample.clone());
    store.last_update.insert(id.clone(), timestamp);

    if let Some(history) = store.history.get_mut(&id) {
      history.push(sample);
    }

    Ok(())
  }

  pub async fn get_value(&self, id: &SensorId) -> Option<SensorSample> {
    let store = self.inner.read().await;
    store.values.get(id).cloned()
  }

  pub async fn get_value_with_timestamp(&self, id: &SensorId) -> Option<(SensorSample, u64)> {
    let store = self.inner.read().await;
    let sample = store.values.get(id).cloned()?;
    let timestamp = store.last_update.get(id).copied()?;
    Some((sample, timestamp))
  }

  pub async fn is_stale(&self, id: &SensorId, now_ms: u64) -> bool {
    let store = self.inner.read().await;
    match store.last_update.get(id) {
      Some(last_update) => {
        now_ms.saturating_sub(*last_update) > store.config.staleness_threshold_ms
      }
      None => true,
    }
  }

  pub async fn get_history(&self, id: &SensorId) -> Option<Vec<SensorSample>> {
    let store = self.inner.read().await;
    store.history.get(id).map(|h| h.iter().cloned().collect())
  }
}

impl Default for SensorStore {
  fn default() -> Self {
    Self::new()
  }
}
