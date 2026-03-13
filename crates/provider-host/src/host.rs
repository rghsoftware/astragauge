use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, RwLock};

use astragauge_sensor_store::{SensorStore, StoreError};
use futures::FutureExt;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::config::HostConfig;
use crate::error::{ProviderError, ProviderResult};
use crate::health::ProviderHealth;
use crate::provider::Provider;

pub struct ProviderEntry {
  provider: Arc<Box<dyn Provider>>,
  task: Option<JoinHandle<()>>,
  health: Arc<RwLock<ProviderHealth>>,
}

pub struct ProviderHost {
  providers: HashMap<String, ProviderEntry>,
  store: Arc<SensorStore>,
  shutdown_token: CancellationToken,
}

impl ProviderHost {
  pub fn new(_config: HostConfig, store: Arc<SensorStore>) -> Self {
    let shutdown_token = CancellationToken::new();
    Self {
      providers: HashMap::new(),
      store,
      shutdown_token,
    }
  }

  pub fn register_provider(&mut self, provider: Arc<Box<dyn Provider>>) -> ProviderResult<()> {
    let manifest = provider.as_ref().manifest();
    let id = manifest.id.clone();

    if self.providers.contains_key(&id) {
      return Err(ProviderError::RegistrationFailed {
        id: id.clone(),
        reason: format!("Provider with id '{}' already registered", id),
      });
    }

    self.providers.insert(
      id,
      ProviderEntry {
        provider: Arc::clone(&provider),
        task: None,
        health: Arc::new(RwLock::new(ProviderHealth::Ok)),
      },
    );

    Ok(())
  }

  pub fn start(&mut self) -> usize {
    let mut count = 0;

    let tasks: Vec<(String, JoinHandle<()>)> = self
      .providers
      .iter()
      .filter(|(_, entry)| entry.task.is_none())
      .map(|(id, entry)| {
        let task = self.spawn_poll_task(
          id.clone(),
          Arc::clone(&entry.provider),
          Arc::clone(&entry.health),
        );
        (id.clone(), task)
      })
      .collect();

    for (id, task) in tasks {
      if let Some(entry) = self.providers.get_mut(&id) {
        entry.task = Some(task);
        count += 1;
      }
    }

    count
  }

  fn spawn_poll_task(
    &self,
    id: String,
    provider: Arc<Box<dyn Provider>>,
    health: Arc<RwLock<ProviderHealth>>,
  ) -> JoinHandle<()> {
    let store = Arc::clone(&self.store);
    let token = self.shutdown_token.clone();
    let poll_interval = provider.as_ref().poll_interval();

    tokio::spawn(async move {
      let discover_result = AssertUnwindSafe(provider.as_ref().discover())
        .catch_unwind()
        .await;

      match discover_result {
        Ok(Ok(descriptors)) => {
          if let Err(e) = Self::register_sensors(&store, &id, descriptors).await {
            tracing::error!("Provider {} sensor registration failed: {:?}", id, e);
          }
        }
        Ok(Err(e)) => {
          tracing::error!("Provider {} sensor discovery failed: {:?}", id, e);
        }
        Err(panic_payload) => {
          Self::handle_panic(&id, &health, panic_payload);
        }
      }

      let mut tick = tokio::time::interval(poll_interval);

      loop {
        tokio::select! {
          _ = tick.tick() => {
            let poll_result = AssertUnwindSafe(provider.as_ref().poll())
              .catch_unwind()
              .await;

            match poll_result {
              Ok(Ok(samples)) => {
                Self::push_samples_with_defensive_registration(
                  &provider, &store, &id, &health, samples
                ).await;
              }
              Ok(Err(e)) => {
                tracing::error!("Provider {} polling failed: {:?}", id, e);
              }
              Err(panic_payload) => {
                Self::handle_panic(&id, &health, panic_payload);
              }
            }
          }
          _ = token.cancelled() => {
            tracing::info!("Provider {} shutdown requested", id);
            break;
          }
        }
      }
    })
  }

  fn handle_panic(
    provider_id: &str,
    health: &Arc<RwLock<ProviderHealth>>,
    payload: Box<dyn std::any::Any + Send>,
  ) {
    let message = if let Some(s) = payload.downcast_ref::<&str>() {
      s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
      s.clone()
    } else {
      "Unknown panic".to_string()
    };

    tracing::error!("Provider {} panicked: {}", provider_id, message);

    if let Ok(mut health_guard) = health.write() {
      *health_guard = ProviderHealth::Error { message };
    }
  }

  async fn register_sensors(
    store: &Arc<SensorStore>,
    provider_id: &str,
    descriptors: Vec<astragauge_domain::SensorDescriptor>,
  ) -> ProviderResult<()> {
    let count = descriptors.len();
    tracing::info!("Provider {} discovered {} sensors", provider_id, count);

    for descriptor in descriptors {
      let sensor_id = descriptor.id.clone();
      match store.register_sensor(descriptor).await {
        Ok(()) => {
          tracing::debug!("Registered sensor {}", sensor_id.as_str());
        }
        Err(e) => {
          tracing::warn!("Failed to register sensor {}: {:?}", sensor_id.as_str(), e);
        }
      }
    }

    Ok(())
  }

  async fn push_samples_with_defensive_registration(
    provider: &Arc<Box<dyn Provider>>,
    store: &Arc<SensorStore>,
    provider_id: &str,
    health: &Arc<RwLock<ProviderHealth>>,
    samples: Vec<astragauge_domain::SensorSample>,
  ) {
    let sample_count = samples.len();

    if let Err(e) = store.push_samples(&samples).await {
      match e {
        StoreError::UnknownSensor { id: unknown_id } => {
          Self::handle_unknown_sensor(
            provider,
            store,
            &unknown_id,
            &samples,
            provider_id,
            health,
            sample_count,
          )
          .await;
        }
        other_error => {
          tracing::error!("Failed to push samples to store: {:?}", other_error);
        }
      }
    } else {
      tracing::trace!(
        "Pushed {} samples from provider {}",
        sample_count,
        provider_id
      );
    }
  }

  async fn handle_unknown_sensor(
    provider: &Arc<Box<dyn Provider>>,
    store: &Arc<SensorStore>,
    unknown_id: &astragauge_domain::SensorId,
    samples: &[astragauge_domain::SensorSample],
    provider_id: &str,
    health: &Arc<RwLock<ProviderHealth>>,
    sample_count: usize,
  ) {
    tracing::warn!(
      "Unknown sensor {} encountered, attempting defensive registration",
      unknown_id.as_str()
    );

    let discover_result = AssertUnwindSafe(provider.as_ref().discover())
      .catch_unwind()
      .await;

    match discover_result {
      Ok(Ok(descriptors)) => {
        if let Some(descriptor) = descriptors.iter().find(|d| &d.id == unknown_id) {
          match store.register_sensor(descriptor.clone()).await {
            Ok(()) => {
              tracing::info!("Defensively registered sensor {}", unknown_id.as_str());
              if let Err(retry_err) = store.push_samples(samples).await {
                tracing::error!(
                  "Failed to push samples after defensive registration: {:?}",
                  retry_err
                );
              } else {
                tracing::debug!(
                  "Pushed {} samples from provider {} after defensive registration",
                  sample_count,
                  provider_id
                );
              }
            }
            Err(reg_err) => {
              tracing::error!(
                "Failed to defensively register sensor {}: {:?}",
                unknown_id.as_str(),
                reg_err
              );
            }
          }
        }
      }
      Ok(Err(e)) => {
        tracing::error!(
          "Failed to re-discover sensors for defensive registration: {:?}",
          e
        );
      }
      Err(panic_payload) => {
        Self::handle_panic(provider_id, health, panic_payload);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::provider::Provider;
  use astragauge_domain::{
    DomainError, ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor,
    SensorId, SensorSample,
  };
  use async_trait::async_trait;
  use std::time::Duration;

  fn make_manifest(id: &str) -> ProviderManifest {
    ProviderManifest {
      id: id.to_string(),
      name: format!("{} provider", id),
      version: "1.0.0".to_string(),
      description: "Test provider".to_string(),
      author: None,
      website: None,
      repository: None,
      license: None,
      tags: None,
      runtime: ">=1.0.0".to_string(),
      capabilities: ProviderCapabilities {
        historical: false,
        high_frequency: false,
        hardware_access: false,
      },
      sensors: SensorCategories { categories: vec![] },
    }
  }

  struct PanickingProvider {
    manifest: ProviderManifest,
  }

  impl PanickingProvider {
    fn new(id: &str) -> Self {
      Self {
        manifest: make_manifest(id),
      }
    }
  }

  #[async_trait]
  impl Provider for PanickingProvider {
    fn manifest(&self) -> &ProviderManifest {
      &self.manifest
    }

    fn poll_interval(&self) -> Duration {
      Duration::from_millis(10)
    }

    async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
      panic!("discover() panicked!")
    }

    async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
      Ok(vec![])
    }

    async fn health(&self) -> ProviderHealth {
      ProviderHealth::Ok
    }

    async fn shutdown(&self) -> ProviderResult<()> {
      Ok(())
    }
  }

  struct PollPanickingProvider {
    manifest: ProviderManifest,
  }

  impl PollPanickingProvider {
    fn new(id: &str) -> Self {
      Self {
        manifest: make_manifest(id),
      }
    }
  }

  #[async_trait]
  impl Provider for PollPanickingProvider {
    fn manifest(&self) -> &ProviderManifest {
      &self.manifest
    }

    fn poll_interval(&self) -> Duration {
      Duration::from_millis(10)
    }

    async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
      Ok(vec![])
    }

    async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
      panic!("poll() panicked!")
    }

    async fn health(&self) -> ProviderHealth {
      ProviderHealth::Ok
    }

    async fn shutdown(&self) -> ProviderResult<()> {
      Ok(())
    }
  }

  struct HealthyProvider {
    manifest: ProviderManifest,
    sensor_id: SensorId,
  }

  impl HealthyProvider {
    fn new(id: &str, sensor_id: &str) -> Result<Self, DomainError> {
      Ok(Self {
        manifest: make_manifest(id),
        sensor_id: SensorId::new(sensor_id.to_string())?,
      })
    }
  }

  #[async_trait]
  impl Provider for HealthyProvider {
    fn manifest(&self) -> &ProviderManifest {
      &self.manifest
    }

    fn poll_interval(&self) -> Duration {
      Duration::from_millis(10)
    }

    async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
      Ok(vec![SensorDescriptor {
        id: self.sensor_id.clone(),
        name: "Test sensor".to_string(),
        category: "test".to_string(),
        unit: "none".to_string(),
        device: None,
        tags: vec![],
      }])
    }

    async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
      Ok(vec![SensorSample {
        sensor_id: self.sensor_id.clone(),
        timestamp_ms: 0,
        value: Some(42.0),
      }])
    }

    async fn health(&self) -> ProviderHealth {
      ProviderHealth::Ok
    }

    async fn shutdown(&self) -> ProviderResult<()> {
      Ok(())
    }
  }

  #[tokio::test]
  #[ignore = "requires panic containment test infrastructure"]
  async fn panic_containment_discover_updates_health() {
    let store = Arc::new(SensorStore::new());
    let config = HostConfig::default();
    let mut host = ProviderHost::new(config, store);

    let provider: Arc<Box<dyn Provider>> = Arc::new(Box::new(PanickingProvider::new("panic-test")));
    host.register_provider(provider).unwrap();

    let health_arc = Arc::clone(&host.providers.get("panic-test").unwrap().health);

    host.start();

    tokio::time::sleep(Duration::from_millis(50)).await;

    let health = health_arc.read().unwrap().clone();
    assert!(matches!(health, ProviderHealth::Error { .. }));
  }

  #[tokio::test]
  #[ignore = "requires panic containment test infrastructure"]
  async fn panic_containment_poll_updates_health() {
    let store = Arc::new(SensorStore::new());
    let config = HostConfig::default();
    let mut host = ProviderHost::new(config, store);

    let provider: Arc<Box<dyn Provider>> =
      Arc::new(Box::new(PollPanickingProvider::new("poll-panic-test")));
    host.register_provider(provider).unwrap();

    let health_arc = Arc::clone(&host.providers.get("poll-panic-test").unwrap().health);

    host.start();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let health = health_arc.read().unwrap().clone();
    assert!(matches!(health, ProviderHealth::Error { ref message } if message.contains("poll()")));
  }

  #[tokio::test]
  #[ignore = "requires panic containment test infrastructure"]
  async fn panic_containment_other_providers_continue() {
    let store = Arc::new(SensorStore::new());
    let config = HostConfig::default();
    let mut host = ProviderHost::new(config, Arc::clone(&store));

    let healthy: Arc<Box<dyn Provider>> = Arc::new(Box::new(
      HealthyProvider::new("healthy", "test.healthy").unwrap(),
    ));
    let panicking: Arc<Box<dyn Provider>> = Arc::new(Box::new(PanickingProvider::new("panicking")));

    host.register_provider(healthy).unwrap();
    host.register_provider(panicking).unwrap();

    let healthy_health = Arc::clone(&host.providers.get("healthy").unwrap().health);
    let panicking_health = Arc::clone(&host.providers.get("panicking").unwrap().health);

    host.start();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let healthy_status = healthy_health.read().unwrap().clone();
    let panicking_status = panicking_health.read().unwrap().clone();

    assert!(matches!(healthy_status, ProviderHealth::Ok));
    assert!(matches!(panicking_status, ProviderHealth::Error { .. }));
  }
}
