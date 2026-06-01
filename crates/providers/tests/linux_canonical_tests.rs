//! Hardware integration test for the LinuxProvider canonical sensor set.
//!
//! Gated `#[cfg(target_os = "linux")]` AND `#[ignore]`: CI runs on ubuntu
//! without amdgpu/k10temp, so this is opt-in. Run explicitly on real hardware:
//!
//!   cargo test -p astragauge-providers --test linux_canonical_tests -- --ignored
//!
//! It builds a LinuxProvider, polls twice (with a short sleep so the CPU
//! utilization delta and GPU values are populated), and asserts the canonical
//! cpu/memory sensors appear with plausible values and the exact canonical unit
//! strings. GPU sensors are best-effort: asserted only if present.

#[cfg(target_os = "linux")]
mod linux_canonical {
  use std::collections::HashMap;
  use std::time::Duration;

  use astragauge_domain::canonical;
  use astragauge_provider_host::Provider;
  use astragauge_providers::LinuxProvider;

  // Canonical (id, unit) pairs that MUST match the MockProvider demo verbatim.
  // Sourced from astragauge_domain::canonical so a drift fails at the source.
  const CANONICAL_CPU_MEM: &[(&str, &str)] = &[
    (canonical::CPU_UTILIZATION, canonical::UNIT_PERCENT),
    (canonical::CPU_CLOCK, canonical::UNIT_MHZ),
    (canonical::CPU_TEMPERATURE, canonical::UNIT_CELSIUS),
    (canonical::MEMORY_USED_PERCENT, canonical::UNIT_PERCENT),
    (canonical::MEMORY_USED, canonical::UNIT_MB),
  ];

  const CANONICAL_GPU: &[(&str, &str)] = &[
    (canonical::GPU_UTILIZATION, canonical::UNIT_PERCENT),
    (canonical::GPU_TEMPERATURE, canonical::UNIT_CELSIUS),
  ];

  #[tokio::test]
  #[ignore]
  async fn canonical_sensors_flow_with_plausible_values() {
    let provider = LinuxProvider::new();

    // Descriptors carry the units; build an id -> unit map.
    let descriptors = provider.discover().await.expect("discover should succeed");
    let units: HashMap<String, String> = descriptors
      .iter()
      .map(|d| (d.id.as_str().to_string(), d.unit.clone()))
      .collect();

    // Two polls so the CPU utilization delta is available.
    let _ = provider.poll().await.expect("first poll should succeed");
    tokio::time::sleep(Duration::from_millis(300)).await;
    let samples = provider.poll().await.expect("second poll should succeed");

    let values: HashMap<String, Option<f64>> = samples
      .iter()
      .map(|s| (s.sensor_id.as_str().to_string(), s.value))
      .collect();

    for (id, value) in &values {
      eprintln!("sample {} = {:?} (unit {:?})", id, value, units.get(id));
    }

    // --- CPU + memory canonical sensors are required on this machine. ---
    for (id, expected_unit) in CANONICAL_CPU_MEM {
      assert!(
        units.get(*id).map(String::as_str) == Some(*expected_unit),
        "descriptor for {} should have unit {:?}, got {:?}",
        id,
        expected_unit,
        units.get(*id)
      );
      let value = values
        .get(*id)
        .copied()
        .flatten()
        .unwrap_or_else(|| panic!("expected a sample value for {}", id));

      match *id {
        "cpu.total.utilization" | "memory.used.percent" => assert!(
          (0.0..=100.0).contains(&value),
          "{} = {} out of 0..=100",
          id,
          value
        ),
        "cpu.clock" => assert!(
          (100.0..=10_000.0).contains(&value),
          "{} = {} MHz implausible",
          id,
          value
        ),
        "cpu.temperature" => assert!(
          (0.0..=120.0).contains(&value),
          "{} = {} °C implausible",
          id,
          value
        ),
        "memory.used" => assert!(value > 0.0, "{} = {} MB should be > 0", id, value),
        _ => {}
      }
    }

    // --- GPU canonical sensors are best-effort: assert only IF present. ---
    for (id, expected_unit) in CANONICAL_GPU {
      let Some(Some(v)) = values.get(*id).copied() else {
        eprintln!("gpu sensor {} not present (no discrete GPU); skipping", id);
        continue;
      };
      assert_eq!(
        units.get(*id).map(String::as_str),
        Some(*expected_unit),
        "descriptor for {} should have unit {:?}",
        id,
        expected_unit
      );
      match *id {
        "gpu.total.utilization" => {
          assert!((0.0..=100.0).contains(&v), "{} = {} out of 0..=100", id, v)
        }
        "gpu.temperature" => {
          assert!((0.0..=120.0).contains(&v), "{} = {} °C implausible", id, v)
        }
        _ => {}
      }
    }
  }
}

#[cfg(not(target_os = "linux"))]
mod linux_canonical {
  #[test]
  #[ignore]
  fn skipped_on_non_linux() {
    println!("LinuxProvider canonical tests are only available on Linux");
  }
}
