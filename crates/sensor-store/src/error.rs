use astragauge_domain::SensorId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreError {
  UnknownSensor { id: SensorId },
  InvalidSample { sensor_id: SensorId, reason: String },
  SubscriptionError { message: String },
}

impl fmt::Display for StoreError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      StoreError::UnknownSensor { id } => {
        write!(f, "Unknown sensor: {}", id.as_str())
      }

      StoreError::InvalidSample { sensor_id, reason } => {
        write!(
          f,
          "Invalid sample for sensor '{}': {}",
          sensor_id.as_str(),
          reason
        )
      }
      StoreError::SubscriptionError { message } => {
        write!(f, "Subscription error: {}", message)
      }
    }
  }
}

impl std::error::Error for StoreError {}

pub type StoreResult<T> = Result<T, StoreError>;
