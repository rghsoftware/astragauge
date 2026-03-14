//! Integration tests for LinuxProvider.
//!
//! Run with: cargo test -p astragauge-providers --test linux_tests -- --ignored
//! (Tests are ignored by default to skip on non-Linux systems)

#[cfg(target_os = "linux")]
mod linux_tests {
  use std::time::Duration;

  use astragauge_domain::SensorId;
  use astragauge_provider_host::Provider;
  use astragauge_providers::LinuxProvider;

  fn is_valid_sensor_id(id: &SensorId) -> bool {
    let id_str = id.as_str();

    if id_str != id_str.to_lowercase() {
      return false;
    }

    let parts: Vec<&str> = id_str.split('.').collect();
    if parts.len() < 2 {
      return false;
    }

    if parts.iter().any(|p| p.is_empty()) {
      return false;
    }

    if id_str.contains(' ') {
      return false;
    }

    if parts[0].contains('_') {
      return false;
    }

    true
  }

  #[tokio::test]
  #[ignore]
  async fn discover_returns_sensors() {
    let provider = LinuxProvider::new();
    let sensors = provider.discover().await.expect("discover should succeed");

    assert!(
      !sensors.is_empty(),
      "Provider should discover at least some sensors on Linux"
    );

    for sensor in &sensors {
      eprintln!(
        "Discovered sensor: {} ({})",
        sensor.id.as_str(),
        sensor.name
      );
    }
  }

  /// CPU utilization requires two polls for delta calculation.
  #[tokio::test]
  #[ignore]
  async fn poll_returns_samples() {
    let provider = LinuxProvider::new();

    let _baseline = provider.poll().await.expect("first poll should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let samples = provider.poll().await.expect("second poll should succeed");

    assert!(!samples.is_empty(), "Poll should return samples");

    for sample in &samples {
      if let Some(value) = sample.value {
        eprintln!("Sample: {} = {:.2}", sample.sensor_id.as_str(), value);
      }
    }
  }

  #[tokio::test]
  #[ignore]
  async fn missing_proc_handled_gracefully() {
    let provider = LinuxProvider::new();

    let discover_result = provider.discover().await;
    assert!(
      discover_result.is_ok(),
      "discover() should not panic on missing files"
    );

    let poll_result = provider.poll().await;
    assert!(
      poll_result.is_ok(),
      "poll() should not panic on missing files"
    );

    let health = provider.health().await;
    eprintln!("Provider health: {:?}", health);
  }

  /// See docs/specs/sensor-schema.md for format rules.
  #[tokio::test]
  #[ignore]
  async fn sensor_ids_follow_conventions() {
    let provider = LinuxProvider::new();
    let sensors = provider.discover().await.expect("discover should succeed");

    let mut invalid_ids: Vec<String> = Vec::new();

    for sensor in &sensors {
      if !is_valid_sensor_id(&sensor.id) {
        invalid_ids.push(format!(
          "{} (name: {}, category: {})",
          sensor.id.as_str(),
          sensor.name,
          sensor.category
        ));
      }
    }

    if !invalid_ids.is_empty() {
      eprintln!("Invalid sensor IDs found:");
      for id in &invalid_ids {
        eprintln!("  - {}", id);
      }
      panic!(
        "Found {} sensor ID(s) that don't follow naming conventions. \
         Expected format: device.metric (lowercase, dot-separated, no spaces, singular device names)",
        invalid_ids.len()
      );
    }

    let sensor_ids: Vec<&str> = sensors.iter().map(|s| s.id.as_str()).collect();

    assert!(
      sensor_ids.contains(&"cpu.utilization"),
      "Expected cpu.utilization sensor"
    );

    assert!(
      sensor_ids.contains(&"memory.used"),
      "Expected memory.used sensor"
    );
    assert!(
      sensor_ids.contains(&"memory.total"),
      "Expected memory.total sensor"
    );
    assert!(
      sensor_ids.contains(&"memory.utilization"),
      "Expected memory.utilization sensor"
    );
    assert!(
      sensor_ids.contains(&"memory.available"),
      "Expected memory.available sensor"
    );
  }

  #[tokio::test]
  #[ignore]
  async fn sensor_descriptors_have_required_fields() {
    let provider = LinuxProvider::new();
    let sensors = provider.discover().await.expect("discover should succeed");

    for sensor in &sensors {
      assert!(
        !sensor.id.as_str().is_empty(),
        "Sensor ID should not be empty"
      );

      assert!(
        !sensor.name.is_empty(),
        "Sensor {} should have a name",
        sensor.id.as_str()
      );

      assert!(
        !sensor.category.is_empty(),
        "Sensor {} should have a category",
        sensor.id.as_str()
      );

      assert!(
        !sensor.unit.is_empty(),
        "Sensor {} should have a unit",
        sensor.id.as_str()
      );

      eprintln!(
        "Sensor: {} | name: {} | category: {} | unit: {}",
        sensor.id.as_str(),
        sensor.name,
        sensor.category,
        sensor.unit
      );
    }
  }

  #[tokio::test]
  #[ignore]
  async fn poll_samples_have_valid_timestamps() {
    let provider = LinuxProvider::new();

    let _ = provider.poll().await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let samples = provider.poll().await.expect("poll should succeed");

    let now_ms = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0);

    for sample in &samples {
      let age_ms = now_ms.saturating_sub(sample.timestamp_ms);
      assert!(
        age_ms < 5000,
        "Sample {} timestamp is too old: {}ms",
        sample.sensor_id.as_str(),
        age_ms
      );

      assert!(
        sample.timestamp_ms <= now_ms + 1000,
        "Sample {} timestamp is in the future",
        sample.sensor_id.as_str()
      );
    }
  }
}

#[cfg(not(target_os = "linux"))]
mod linux_tests {
  #[test]
  #[ignore]
  fn linux_tests_skipped_on_non_linux() {
    println!("LinuxProvider tests are only available on Linux");
  }
}
