use std::collections::HashMap;
use std::sync::Arc;

use astragauge_sensor_store::SensorStore;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::config::HostConfig;
use crate::error::{ProviderError, ProviderResult};
use crate::provider::Provider;

pub struct ProviderEntry {
  provider: Arc<Box<dyn Provider>>,
  task: Option<JoinHandle<()>>,
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

    for (id, mut entry) in self.providers.iter_mut() {
      if entry.task.is_none() {
        let _task = self.spawn_poll_task(id.clone(), Arc::clone(&entry.provider));
        entry.task = Some(_task);
        count += 1;
      }
    }

    count
  }

  fn spawn_poll_task(&mut self, id: String, provider: Arc<Box<dyn Provider>>) -> JoinHandle<()> {
    let store = Arc::clone(&self.store);
    let token = self.shutdown_token.clone();
    let poll_interval = provider.as_ref().poll_interval();

    tokio::spawn(async move {
      let mut tick = tokio::time::interval(poll_interval);

      loop {
        tokio::select! {
          _ = tick.tick() => {
            match provider.as_ref().poll().await {
              Ok(samples) => {
                if let Err(e) = store.push_samples(&samples).await {
                  tracing::error!("Failed to push samples to store: {:?}", e);
                }
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
}
