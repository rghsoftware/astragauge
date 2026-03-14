use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
#[serde(rename_all = "snake_case")]
pub enum ProviderError {
  #[error("Provider discovery failed: {message}")]
  DiscoveryFailed { message: String },
  #[error("Provider poll failed: {message}")]
  PollFailed { message: String },
  #[error("Provider shutdown failed: {message}")]
  ShutdownFailed { message: String },
  #[error("Provider registration failed for '{id}': {reason}")]
  RegistrationFailed { id: String, reason: String },
  #[error("Invalid manifest: {reason}")]
  InvalidManifest { reason: String },
}

pub type ProviderResult<T> = Result<T, ProviderError>;
