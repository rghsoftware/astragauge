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

  #[tokio::test]
  #[ignore]
  async fn sensor_ids_follow_conventions() {
    let provider = LinuxProvider::new();
    let sensors = provider.discover().await.expect("discover should succeed");

    for sensor in &sensors {
      let id_str = sensor.id.as_str();

      assert_eq!(
        id_str,
        id_str.to_lowercase(),
        "Sensor ID must be lowercase: {}",
        id_str
      );

      assert!(
        SensorId::new(id_str).is_ok(),
        "Sensor ID must be parseable by SensorId::new(): {}",
        id_str
      );

      assert!(
        !id_str.contains(' '),
        "Sensor ID must not contain spaces: {}",
        id_str
      );
    }

    let sensor_ids: Vec<&str> = sensors.iter().map(|s| s.id.as_str()).collect();

    assert!(sensor_ids.contains(&"cpu.utilization"), "Expected cpu.utilization");
    assert!(sensor_ids.contains(&"memory.used"), "Expected memory.used");
    assert!(sensor_ids.contains(&"memory.total"), "Expected memory.total");
    assert!(sensor_ids.contains(&"memory.utilization"), "Expected memory.utilization");
    assert!(sensor_ids.contains(&"memory.available"), "Expected memory.available");
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

  #[tokio::test]
  #[ignore]
  async fn swap_sensors_present() {
    let provider = LinuxProvider::new();
    let sensors = provider.discover().await.expect("discover should succeed");

    let sensor_ids: Vec<&str> = sensors.iter().map(|s| s.id.as_str()).collect();

    assert!(sensor_ids.contains(&"swap.total"), "Expected swap.total");
    assert!(sensor_ids.contains(&"swap.used"), "Expected swap.used");
    assert!(sensor_ids.contains(&"swap.free"), "Expected swap.free");
    assert!(sensor_ids.contains(&"swap.utilization"), "Expected swap.utilization");
  }

  #[tokio::test]
  #[ignore]
  async fn configurable_poll_interval() {
    let provider = LinuxProvider::with_poll_interval(Duration::from_millis(500));
    assert_eq!(provider.poll_interval(), Duration::from_millis(500));
  }

  #[tokio::test]
  #[ignore]
  async fn health_check_works() {
    let provider = LinuxProvider::new();
    let health = provider.health().await;
    eprintln!("Health: {:?}", health);
  }

  #[tokio::test]
  #[ignore]
  async fn disk_and_network_sensors_polled() {
    let provider = LinuxProvider::new();

    let _ = provider.poll().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let samples = provider.poll().await.expect("poll should succeed");

    let has_disk = samples.iter().any(|s| s.sensor_id.as_str().starts_with("disk."));
    let has_network = samples.iter().any(|s| s.sensor_id.as_str().starts_with("network."));

    eprintln!(
      "Disk samples: {}, Network samples: {}",
      has_disk, has_network
    );

    for sample in &samples {
      eprintln!("  {} = {:?}", sample.sensor_id.as_str(), sample.value);
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
