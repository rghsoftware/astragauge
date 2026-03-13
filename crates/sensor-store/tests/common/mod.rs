use astragauge_domain::{SensorDescriptor, SensorId, SensorSample};

pub fn make_descriptor(id: &str) -> SensorDescriptor {
  SensorDescriptor {
    id: SensorId::new(id).unwrap(),
    name: format!("{} sensor", id),
    category: "test".to_string(),
    unit: "unit".to_string(),
    device: None,
    tags: vec![],
  }
}

pub fn make_sample(id: &str, timestamp_ms: u64, value: Option<f64>) -> SensorSample {
  SensorSample {
    sensor_id: SensorId::new(id).unwrap(),
    timestamp_ms,
    value,
  }
}
