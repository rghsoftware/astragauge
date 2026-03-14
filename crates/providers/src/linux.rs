//! Linux system provider - reads from /proc, /sys/class/hwmon for CPU, memory, and temperature sensors.

#[cfg(target_os = "linux")]
use async_trait::async_trait;
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorId,
  SensorSample,
};
#[cfg(target_os = "linux")]
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

/// CPU statistics from /proc/stat for utilization calculation.
/// Stores cumulative time values in jiffies (typically 1/100 second).
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, Default)]
struct CpuStats {
  user: u64,
  nice: u64,
  system: u64,
  idle: u64,
  iowait: u64,
  irq: u64,
  softirq: u64,
  steal: u64,
  guest: u64,
  guest_nice: u64,
}

#[cfg(target_os = "linux")]
impl CpuStats {
  /// Calculate CPU utilization percentage from current and previous stats.
  /// Returns None if delta is zero (no change between readings).
  fn utilization_from(prev: CpuStats, curr: CpuStats) -> Option<f64> {
    let prev_idle = prev.idle + prev.iowait;
    let curr_idle = curr.idle + curr.iowait;

    let prev_total = prev.user
      + prev.nice
      + prev.system
      + prev.idle
      + prev.iowait
      + prev.irq
      + prev.softirq
      + prev.steal
      + prev.guest
      + prev.guest_nice;

    let curr_total = curr.user
      + curr.nice
      + curr.system
      + curr.idle
      + curr.iowait
      + curr.irq
      + curr.softirq
      + curr.steal
      + curr.guest
      + curr.guest_nice;

    let delta_total = curr_total.saturating_sub(prev_total);
    let delta_idle = curr_idle.saturating_sub(prev_idle);

    if delta_total == 0 {
      return None;
    }

    let utilization = ((delta_total - delta_idle) as f64 / delta_total as f64) * 100.0;
    Some(utilization.clamp(0.0, 100.0))
  }
}

#[cfg(target_os = "linux")]
pub struct LinuxProvider {
  manifest: ProviderManifest,
  sensors: Vec<SensorDescriptor>,
  /// Previous CPU stats for utilization delta calculation.
  /// Uses Arc<Mutex> for interior mutability since poll() takes &self.
  prev_cpu_stats: Arc<Mutex<Option<CpuStats>>>,
}

#[cfg(target_os = "linux")]
impl LinuxProvider {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    tracing::info!("Initializing Linux provider");
    let sensors = Self::discover_sensors();
    tracing::info!("Linux provider discovered {} sensors", sensors.len());
    Self {
      manifest: linux_manifest(),
      sensors,
      prev_cpu_stats: Arc::new(Mutex::new(None)),
    }
  }

  fn discover_sensors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    match Self::discover_cpu_sensors() {
      Ok(cpu_sensors) => {
        tracing::debug!("Discovered {} CPU sensors", cpu_sensors.len());
        sensors.extend(cpu_sensors);
      }
      Err(e) => {
        tracing::warn!("Failed to discover CPU sensors: {}", e);
      }
    }

    match Self::discover_memory_sensors() {
      Ok(mem_sensors) => {
        tracing::debug!("Discovered {} memory sensors", mem_sensors.len());
        sensors.extend(mem_sensors);
      }
      Err(e) => {
        tracing::warn!("Failed to discover memory sensors: {}", e);
      }
    }

    match Self::discover_hwmon_sensors() {
      Ok(hwmon_sensors) => {
        tracing::debug!(
          "Discovered {} hwmon temperature sensors",
          hwmon_sensors.len()
        );
        sensors.extend(hwmon_sensors);
      }
      Err(e) => {
        tracing::warn!("Failed to discover hwmon sensors: {}", e);
      }
    }

    sensors
  }

  fn discover_cpu_sensors() -> std::io::Result<Vec<SensorDescriptor>> {
    let _content = fs::read_to_string("/proc/cpuinfo")?;
    let mut sensors = Vec::new();

    if let Ok(id) = SensorId::new("cpu.utilization") {
      sensors.push(SensorDescriptor {
        id,
        name: "CPU Utilization".to_string(),
        category: "cpu".to_string(),
        unit: "percent".to_string(),
        device: None,
        tags: vec!["cpu".to_string()],
      });
    }

    Ok(sensors)
  }

  fn discover_memory_sensors() -> std::io::Result<Vec<SensorDescriptor>> {
    let _content = fs::read_to_string("/proc/meminfo")?;
    let mut sensors = Vec::new();

    if let Ok(id) = SensorId::new("memory.used") {
      sensors.push(SensorDescriptor {
        id,
        name: "Memory Used".to_string(),
        category: "memory".to_string(),
        unit: "bytes".to_string(),
        device: None,
        tags: vec!["memory".to_string()],
      });
    }

    if let Ok(id) = SensorId::new("memory.total") {
      sensors.push(SensorDescriptor {
        id,
        name: "Memory Total".to_string(),
        category: "memory".to_string(),
        unit: "bytes".to_string(),
        device: None,
        tags: vec!["memory".to_string()],
      });
    }

    if let Ok(id) = SensorId::new("memory.utilization") {
      sensors.push(SensorDescriptor {
        id,
        name: "Memory Utilization".to_string(),
        category: "memory".to_string(),
        unit: "percent".to_string(),
        device: None,
        tags: vec!["memory".to_string()],
      });
    }

    if let Ok(id) = SensorId::new("memory.available") {
      sensors.push(SensorDescriptor {
        id,
        name: "Memory Available".to_string(),
        category: "memory".to_string(),
        unit: "bytes".to_string(),
        device: None,
        tags: vec!["memory".to_string()],
      });
    }

    Ok(sensors)
  }

  fn discover_hwmon_sensors() -> std::io::Result<Vec<SensorDescriptor>> {
    let mut sensors = Vec::new();
    let hwmon_path = Path::new("/sys/class/hwmon");

    if !hwmon_path.exists() {
      tracing::debug!("hwmon path does not exist: {}", hwmon_path.display());
      return Ok(sensors);
    }

    let entries = fs::read_dir(hwmon_path)?;
    for entry in entries {
      let entry = entry?;
      let hwmon_dir = entry.path();

      let name = fs::read_to_string(hwmon_dir.join("name"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_lowercase();

      let temp_sensors = Self::find_temp_sensors(&hwmon_dir, &name);
      sensors.extend(temp_sensors);
    }

    Ok(sensors)
  }

  fn find_temp_sensors(hwmon_dir: &Path, device_name: &str) -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    if let Ok(entries) = fs::read_dir(hwmon_dir) {
      for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if !(name.ends_with("_input") && name.starts_with("temp")) {
          continue;
        }

        let index: String = name
          .chars()
          .skip(4)
          .take_while(|c| c.is_ascii_digit())
          .collect();

        if index.is_empty() {
          continue;
        }

        let label = fs::read_to_string(hwmon_dir.join(format!("temp{}_label", index)))
          .ok()
          .map(|s| s.trim().to_string());

        let sensor_id = if device_name.contains("coretemp")
          || device_name.contains("cpu")
          || device_name.contains("k10temp")
          || device_name.contains("k8temp")
        {
          format!("cpu.{}.temperature", device_name)
        } else if device_name.contains("gpu") || device_name.contains("nvidia") {
          format!("gpu.{}.temperature", device_name)
        } else {
          format!("{}.temperature", device_name.replace(' ', "_"))
        };

        if let Ok(id) = SensorId::new(&sensor_id) {
          let display_name = label.unwrap_or_else(|| format!("{} Temperature", device_name));

          sensors.push(SensorDescriptor {
            id,
            name: display_name,
            category: "temperature".to_string(),
            unit: "celsius".to_string(),
            device: Some(device_name.to_string()),
            tags: vec!["thermal".to_string(), device_name.to_string()],
          });
        }
      }
    }

    sensors
  }

  /// Get current timestamp in milliseconds since UNIX epoch.
  fn current_timestamp_ms() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0)
  }

  /// Parse /proc/stat first line (aggregate CPU stats).
  /// Format: cpu  user nice system idle iowait irq softirq steal guest guest_nice
  fn read_cpu_stats() -> Option<CpuStats> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let first_line = content.lines().next()?;

    // Skip "cpu" prefix and parse values
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 8 || parts[0] != "cpu" {
      return None;
    }

    let parse_u64 =
      |idx: usize| -> u64 { parts.get(idx).and_then(|s| s.parse().ok()).unwrap_or(0) };

    Some(CpuStats {
      user: parse_u64(1),
      nice: parse_u64(2),
      system: parse_u64(3),
      idle: parse_u64(4),
      iowait: parse_u64(5),
      irq: parse_u64(6),
      softirq: parse_u64(7),
      steal: parse_u64(8),
      guest: parse_u64(9),
      guest_nice: parse_u64(10),
    })
  }

  /// Poll CPU utilization sensor.
  /// Requires previous stats for delta calculation.
  fn poll_cpu(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let current_stats = match Self::read_cpu_stats() {
      Some(stats) => stats,
      None => {
        tracing::warn!("Failed to read /proc/stat for CPU utilization");
        return;
      }
    };

    // Calculate utilization if we have previous stats
    let mut prev_guard = self.prev_cpu_stats.lock().unwrap();
    if let Some(prev_stats) = prev_guard.as_ref() {
      if let Some(utilization) = CpuStats::utilization_from(*prev_stats, current_stats) {
        if let Ok(id) = SensorId::new("cpu.utilization") {
          samples.push(SensorSample {
            sensor_id: id,
            timestamp_ms,
            value: Some(utilization),
          });
        }
      }
    }

    // Store current stats for next poll
    *prev_guard = Some(current_stats);
  }

  /// Parse /proc/meminfo and return key-value pairs in kB.
  fn read_meminfo() -> Option<HashMap<String, u64>> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut meminfo = HashMap::new();

    for line in content.lines() {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() >= 2 {
        // Remove trailing colon from key (e.g., "MemTotal:")
        let key = parts[0].trim_end_matches(':');
        // Value is in kB, convert to bytes
        if let Ok(kb) = parts[1].parse::<u64>() {
          meminfo.insert(key.to_string(), kb * 1024);
        }
      }
    }

    Some(meminfo)
  }

  /// Poll memory sensors from /proc/meminfo.
  fn poll_memory(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let meminfo = match Self::read_meminfo() {
      Some(info) => info,
      None => {
        tracing::warn!("Failed to read /proc/meminfo");
        return;
      }
    };

    let mem_total = meminfo.get("MemTotal").copied().unwrap_or(0);
    let mem_available = meminfo.get("MemAvailable").copied().unwrap_or(0);
    let mem_free = meminfo.get("MemFree").copied().unwrap_or(0);
    let buffers = meminfo.get("Buffers").copied().unwrap_or(0);
    let cached = meminfo.get("Cached").copied().unwrap_or(0);

    // Used memory = Total - Available (more accurate than Total - Free)
    let mem_used = mem_total.saturating_sub(mem_available);

    // Memory utilization percentage
    let mem_utilization = if mem_total > 0 {
      Some((mem_used as f64 / mem_total as f64) * 100.0)
    } else {
      None
    };

    // Push samples
    if let Ok(id) = SensorId::new("memory.total") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: Some(mem_total as f64),
      });
    }

    if let Ok(id) = SensorId::new("memory.used") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: Some(mem_used as f64),
      });
    }

    if let Ok(id) = SensorId::new("memory.available") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: Some(mem_available as f64),
      });
    }

    if let Ok(id) = SensorId::new("memory.utilization") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: mem_utilization,
      });
    }

    // Log additional info for debugging
    tracing::trace!(
      "Memory: total={}MB, used={}MB, available={}MB, free={}MB, buffers={}MB, cached={}MB",
      mem_total / 1024 / 1024,
      mem_used / 1024 / 1024,
      mem_available / 1024 / 1024,
      mem_free / 1024 / 1024,
      buffers / 1024 / 1024,
      cached / 1024 / 1024
    );
  }

  /// Poll temperature sensors from /sys/class/hwmon.
  fn poll_temperatures(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let hwmon_path = Path::new("/sys/class/hwmon");

    if !hwmon_path.exists() {
      tracing::trace!("hwmon path does not exist");
      return;
    }

    let entries = match fs::read_dir(hwmon_path) {
      Ok(e) => e,
      Err(e) => {
        tracing::warn!("Failed to read hwmon directory: {}", e);
        return;
      }
    };

    for entry in entries.flatten() {
      let hwmon_dir = entry.path();

      let device_name = match fs::read_to_string(hwmon_dir.join("name")) {
        Ok(name) => name.trim().to_lowercase(),
        Err(_) => continue,
      };

      // Determine sensor ID prefix based on device name
      let sensor_prefix = if device_name.contains("coretemp")
        || device_name.contains("cpu")
        || device_name.contains("k10temp")
        || device_name.contains("k8temp")
      {
        "cpu"
      } else if device_name.contains("gpu") || device_name.contains("nvidia") {
        "gpu"
      } else {
        &device_name.replace(' ', "_")
      };

      // Find and read temperature input files
      if let Ok(dir_entries) = fs::read_dir(&hwmon_dir) {
        for file_entry in dir_entries.flatten() {
          let file_name = file_entry.file_name();
          let name = file_name.to_string_lossy();

          if !(name.ends_with("_input") && name.starts_with("temp")) {
            continue;
          }

          // Read temperature value (in millidegrees Celsius)
          let temp_content = match fs::read_to_string(file_entry.path()) {
            Ok(c) => c,
            Err(e) => {
              tracing::trace!("Failed to read {}: {}", file_entry.path().display(), e);
              continue;
            }
          };

          let temp_mc: i64 = match temp_content.trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
          };

          // Convert millidegrees to Celsius
          let temp_c = temp_mc as f64 / 1000.0;

          // Create sensor ID
          let sensor_id_str = format!("{}.temperature", sensor_prefix);
          if let Ok(id) = SensorId::new(&sensor_id_str) {
            samples.push(SensorSample {
              sensor_id: id,
              timestamp_ms,
              value: Some(temp_c),
            });
          }
        }
      }
    }
  }
}

#[cfg(target_os = "linux")]
fn linux_manifest() -> ProviderManifest {
  ProviderManifest {
    id: "linux.provider".to_string(),
    name: "Linux System Provider".to_string(),
    version: env!("CARGO_PKG_VERSION").to_string(),
    description: "System metrics from Linux kernel".to_string(),
    author: Some("AstraGauge".to_string()),
    website: None,
    repository: None,
    license: Some("MIT".to_string()),
    tags: Some(vec!["linux".to_string(), "system".to_string()]),
    runtime: ">=0.1.0".to_string(),
    capabilities: ProviderCapabilities {
      historical: false,
      high_frequency: false,
      hardware_access: true,
    },
    sensors: SensorCategories {
      categories: vec![
        "cpu".to_string(),
        "memory".to_string(),
        "temperature".to_string(),
      ],
    },
  }
}

#[cfg(target_os = "linux")]
#[async_trait]
impl Provider for LinuxProvider {
  fn manifest(&self) -> &ProviderManifest {
    &self.manifest
  }

  fn poll_interval(&self) -> std::time::Duration {
    std::time::Duration::from_millis(1000)
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    Ok(self.sensors.clone())
  }

  async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
    let mut samples = Vec::new();
    let timestamp_ms = Self::current_timestamp_ms();

    // Poll each sensor category
    self.poll_cpu(&mut samples, timestamp_ms);
    self.poll_memory(&mut samples, timestamp_ms);
    self.poll_temperatures(&mut samples, timestamp_ms);

    tracing::trace!("Linux provider polled {} samples", samples.len());
    Ok(samples)
  }

  async fn health(&self) -> ProviderHealth {
    ProviderHealth::Ok
  }

  async fn shutdown(&self) -> ProviderResult<()> {
    Ok(())
  }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cpu_stats_default() {
    let stats = CpuStats::default();
    assert_eq!(stats.user, 0);
    assert_eq!(stats.idle, 0);
  }

  #[test]
  fn test_calculate_cpu_utilization_basic() {
    let prev = CpuStats {
      user: 100,
      nice: 0,
      system: 50,
      idle: 800,
      iowait: 10,
      irq: 5,
      softirq: 5,
      steal: 0,
      guest: 0,
      guest_nice: 0,
    };

    let curr = CpuStats {
      user: 150, // +50
      nice: 0,
      system: 75, // +25
      idle: 900,  // +90
      iowait: 15, // +5
      irq: 7,     // +2
      softirq: 6, // +1
      steal: 0,
      guest: 0,
      guest_nice: 0,
    };

    // Total delta = 50 + 0 + 25 + 100 + 5 + 2 + 1 = 183
    // Idle delta = 100 + 5 = 105
    // Active delta = 183 - 105 = 78
    // Utilization = 78 / 183 * 100 = 42.62%
    let utilization = CpuStats::utilization_from(prev, curr);
    assert!(utilization.is_some());

    let util = utilization.unwrap();
    assert!(util > 42.0 && util < 43.0, "Expected ~42.6%, got {}", util);
  }

  #[test]
  fn test_calculate_cpu_utilization_zero_delta() {
    let stats = CpuStats {
      user: 100,
      nice: 0,
      system: 50,
      idle: 800,
      iowait: 10,
      irq: 5,
      softirq: 5,
      steal: 0,
      guest: 0,
      guest_nice: 0,
    };

    // Same stats should return None (no change)
    let utilization = CpuStats::utilization_from(stats, stats);
    assert!(utilization.is_none());
  }

  #[test]
  fn test_calculate_cpu_utilization_clamp() {
    let prev = CpuStats {
      user: 100,
      idle: 100,
      ..CpuStats::default()
    };

    // Edge case: all active, no idle
    let curr = CpuStats {
      user: 200, // +100 active
      idle: 100, // 0 idle
      ..CpuStats::default()
    };

    let utilization = CpuStats::utilization_from(prev, curr);
    assert!(utilization.is_some());
    assert!((utilization.unwrap() - 100.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_current_timestamp_ms() {
    let ts = LinuxProvider::current_timestamp_ms();
    // Should be a reasonable timestamp (after year 2020)
    assert!(ts > 1577836800000); // Jan 1, 2020 in ms
  }
}
