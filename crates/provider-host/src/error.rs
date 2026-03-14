use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderError {
  DiscoveryFailed { message: String },
  PollFailed { message: String },
  ShutdownFailed { message: String },
  RegistrationFailed { id: String, reason: String },
  InvalidManifest { reason: String },
}

impl fmt::Display for ProviderError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ProviderError::DiscoveryFailed { message } => {
        write!(f, "Provider discovery failed: {}", message)
      }
      ProviderError::PollFailed { message } => {
        write!(f, "Provider poll failed: {}", message)
      }
      ProviderError::ShutdownFailed { message } => {
        write!(f, "Provider shutdown failed: {}", message)
      }
      ProviderError::RegistrationFailed { id, reason } => {
        write!(f, "Provider registration failed for '{}': {}", id, reason)
      }
      ProviderError::InvalidManifest { reason } => {
        write!(f, "Invalid manifest: {}", reason)
      }
    }
  }
}

impl std::error::Error for ProviderError {}

pub type ProviderResult<T> = Result<T, ProviderError>;
