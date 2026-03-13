use std::collections::HashMap;
use std::sync::Arc;

use astragauge_sensor_store::{SensorStore, StoreError};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::config::HostConfig;
use crate::error::{ProviderError, ProviderResult};
use crate::health::ProviderHealth;
use crate::provider::Provider;

pub struct ProviderEntry {
  provider: Arc<Box<dyn Provider>>,
  task: Option<JoinHandle<()>>,
  #[allow(dead_code)]
  health: ProviderHealth,
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
        health: ProviderHealth::Ok,
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
        let task = self.spawn_poll_task(id.clone(), Arc::clone(&entry.provider));
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

  fn spawn_poll_task(&self, id: String, provider: Arc<Box<dyn Provider>>) -> JoinHandle<()> {
    let store = Arc::clone(&self.store);
    let token = self.shutdown_token.clone();
    let poll_interval = provider.as_ref().poll_interval();

    tokio::spawn(async move {
      if let Err(e) = Self::discover_and_register(&provider, &store, &id).await {
        tracing::error!("Provider {} sensor discovery failed: {:?}", id, e);
      }

      let mut tick = tokio::time::interval(poll_interval);

      loop {
        tokio::select! {
          _ = tick.tick() => {
            match provider.as_ref().poll().await {
              Ok(samples) => {
                Self::push_samples_with_defensive_registration(
                  &provider, &store, &id, samples
                ).await;
              }
              Err(e) => {
                tracing::error!("Provider {} polling failed: {:?}", id, e);
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

  async fn discover_and_register(
    provider: &Arc<Box<dyn Provider>>,
    store: &Arc<SensorStore>,
    provider_id: &str,
  ) -> ProviderResult<()> {
    let descriptors = provider.as_ref().discover().await?;
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
    sample_count: usize,
  ) {
    tracing::warn!(
      "Unknown sensor {} encountered, attempting defensive registration",
      unknown_id.as_str()
    );

    match provider.as_ref().discover().await {
      Ok(descriptors) => {
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
      Err(e) => {
        tracing::error!(
          "Failed to re-discover sensors for defensive registration: {:?}",
          e
        );
      }
    }
  }
}
