//! Canonical sensor identifiers and unit strings shared across providers and
//! the default instrument panel.
//!
//! These are the contract between providers and the UI: any provider that wants
//! to populate the default panel MUST emit these exact ids with these exact unit
//! strings. Referencing these constants (rather than re-typing the literals)
//! keeps the MockProvider and LinuxProvider interchangeable and makes id/unit
//! divergence impossible at the source.

/// CPU total utilization, 0–100. Unit: [`UNIT_PERCENT`].
pub const CPU_UTILIZATION: &str = "cpu.total.utilization";
/// Mean CPU core clock. Unit: [`UNIT_MHZ`].
pub const CPU_CLOCK: &str = "cpu.clock";
/// CPU package temperature. Unit: [`UNIT_CELSIUS`].
pub const CPU_TEMPERATURE: &str = "cpu.temperature";
/// Discrete-GPU utilization, 0–100. Unit: [`UNIT_PERCENT`].
pub const GPU_UTILIZATION: &str = "gpu.total.utilization";
/// Discrete-GPU temperature. Unit: [`UNIT_CELSIUS`].
pub const GPU_TEMPERATURE: &str = "gpu.temperature";
/// Memory used as a percentage of total, 0–100. Unit: [`UNIT_PERCENT`].
pub const MEMORY_USED_PERCENT: &str = "memory.used.percent";
/// Memory used in megabytes. Unit: [`UNIT_MB`].
pub const MEMORY_USED: &str = "memory.used";

/// Percentage (`%`) — used by the `*.utilization` / `*.used.percent` sensors.
pub const UNIT_PERCENT: &str = "%";
/// Megahertz (`MHz`) — used by [`CPU_CLOCK`].
pub const UNIT_MHZ: &str = "MHz";
/// Degrees Celsius (`°C`) — used by the `*.temperature` sensors.
pub const UNIT_CELSIUS: &str = "°C";
/// Megabytes (`MB`) — used by [`MEMORY_USED`] and the memory extras.
pub const UNIT_MB: &str = "MB";

/// All 7 canonical sensor ids, for iteration in tests and discovery checks.
pub const ALL: [&str; 7] = [
  CPU_UTILIZATION,
  CPU_CLOCK,
  CPU_TEMPERATURE,
  GPU_UTILIZATION,
  GPU_TEMPERATURE,
  MEMORY_USED_PERCENT,
  MEMORY_USED,
];

/// The canonical `(id, unit)` pairing every provider must honor. Tests assert
/// their descriptors against this so a unit-string drift fails in CI.
pub const ID_UNITS: [(&str, &str); 7] = [
  (CPU_UTILIZATION, UNIT_PERCENT),
  (CPU_CLOCK, UNIT_MHZ),
  (CPU_TEMPERATURE, UNIT_CELSIUS),
  (GPU_UTILIZATION, UNIT_PERCENT),
  (GPU_TEMPERATURE, UNIT_CELSIUS),
  (MEMORY_USED_PERCENT, UNIT_PERCENT),
  (MEMORY_USED, UNIT_MB),
];

#[cfg(test)]
mod tests {
  use super::*;
  use crate::SensorId;

  #[test]
  fn all_canonical_ids_are_valid_sensor_ids() {
    for id in ALL {
      SensorId::new(id).unwrap_or_else(|e| panic!("canonical id {id:?} is invalid: {e}"));
    }
  }

  #[test]
  fn id_units_covers_every_canonical_id() {
    for id in ALL {
      assert!(
        ID_UNITS.iter().any(|(pair_id, _)| *pair_id == id),
        "canonical id {id:?} missing from ID_UNITS"
      );
    }
  }

  /// Pins the literal id/unit strings. This is the test that actually catches a
  /// rename: providers reference these constants, and the frontend
  /// `demo.panel.json` binds by these exact id strings, so changing a value here
  /// silently blanks a widget or drifts a unit. Asserting against the constants
  /// (as the provider tests do) is tautological — the literals must be pinned.
  #[test]
  fn canonical_strings_are_stable() {
    assert_eq!(CPU_UTILIZATION, "cpu.total.utilization");
    assert_eq!(CPU_CLOCK, "cpu.clock");
    assert_eq!(CPU_TEMPERATURE, "cpu.temperature");
    assert_eq!(GPU_UTILIZATION, "gpu.total.utilization");
    assert_eq!(GPU_TEMPERATURE, "gpu.temperature");
    assert_eq!(MEMORY_USED_PERCENT, "memory.used.percent");
    assert_eq!(MEMORY_USED, "memory.used");
    assert_eq!(
      (UNIT_PERCENT, UNIT_MHZ, UNIT_CELSIUS, UNIT_MB),
      ("%", "MHz", "°C", "MB")
    );
  }
}
