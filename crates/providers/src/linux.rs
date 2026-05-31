//! Linux system provider - reads from /proc, /sys/class/hwmon for CPU, memory,
//! swap, disk, network, and temperature sensors.

#[cfg(target_os = "linux")]
use async_trait::async_trait;
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};
#[cfg(target_os = "linux")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorId,
  SensorSample,
};
#[cfg(target_os = "linux")]
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

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

  fn parse_from_line(line: &str) -> Option<CpuStats> {
    let parts: Vec<&str> = line.split_whitespace().collect();
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
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, Default)]
struct DiskStats {
  sectors_read: u64,
  sectors_written: u64,
}

#[cfg(target_os = "linux")]
impl DiskStats {
  fn parse_from_line(line: &str) -> Option<(String, DiskStats)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 10 {
      return None;
    }

    let name = parts.get(2)?.to_string();

    let parse_u64 =
      |idx: usize| -> u64 { parts.get(idx).and_then(|s| s.parse().ok()).unwrap_or(0) };

    Some((
      name,
      DiskStats {
        sectors_read: parse_u64(5),
        sectors_written: parse_u64(9),
      },
    ))
  }

  fn read_bytes_delta(prev: DiskStats, curr: DiskStats) -> u64 {
    curr
      .sectors_read
      .saturating_sub(prev.sectors_read)
      .saturating_mul(512)
  }

  fn write_bytes_delta(prev: DiskStats, curr: DiskStats) -> u64 {
    curr
      .sectors_written
      .saturating_sub(prev.sectors_written)
      .saturating_mul(512)
  }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, Default)]
struct NetStats {
  rx_bytes: u64,
  tx_bytes: u64,
}

#[cfg(target_os = "linux")]
impl NetStats {
  fn parse_from_line(line: &str) -> Option<(String, NetStats)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 10 {
      return None;
    }

    let iface_with_colon = parts.first()?;
    if !iface_with_colon.contains(':') {
      return None;
    }

    let name = iface_with_colon.trim_end_matches(':').to_string();
    if name == "lo" {
      return None;
    }

    let parse_u64 =
      |idx: usize| -> u64 { parts.get(idx).and_then(|s| s.parse().ok()).unwrap_or(0) };

    Some((
      name,
      NetStats {
        rx_bytes: parse_u64(1),
        tx_bytes: parse_u64(9),
      },
    ))
  }

  fn rx_delta(prev: NetStats, curr: NetStats) -> u64 {
    curr.rx_bytes.saturating_sub(prev.rx_bytes)
  }

  fn tx_delta(prev: NetStats, curr: NetStats) -> u64 {
    curr.tx_bytes.saturating_sub(prev.tx_bytes)
  }
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct PreviousStats {
  cpu: Option<CpuStats>,
  disks: HashMap<String, DiskStats>,
  networks: HashMap<String, NetStats>,
  timestamp_ms: u64,
}

#[cfg(target_os = "linux")]
pub struct LinuxProvider {
  manifest: ProviderManifest,
  sensors: Vec<SensorDescriptor>,
  prev_stats: Arc<Mutex<PreviousStats>>,
  poll_interval: Duration,
}

#[cfg(target_os = "linux")]
impl Default for LinuxProvider {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(target_os = "linux")]
impl LinuxProvider {
  pub fn new() -> Self {
    Self::with_poll_interval(Duration::from_millis(1000))
  }

  pub fn with_poll_interval(poll_interval: Duration) -> Self {
    tracing::info!("Initializing Linux provider");
    let sensors = Self::discover_sensors_sync();
    tracing::info!("Linux provider discovered {} sensors", sensors.len());
    Self {
      manifest: linux_manifest(),
      sensors,
      prev_stats: Arc::new(Mutex::new(PreviousStats::default())),
      poll_interval,
    }
  }

  fn discover_sensors_sync() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    sensors.extend(Self::cpu_sensor_descriptors());
    sensors.extend(Self::memory_sensor_descriptors());
    sensors.extend(Self::swap_sensor_descriptors());
    sensors.extend(Self::disk_sensor_descriptors());
    sensors.extend(Self::network_sensor_descriptors());
    sensors.extend(Self::hwmon_sensor_descriptors());

    sensors
  }

  fn cpu_sensor_descriptors() -> Vec<SensorDescriptor> {
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
    sensors
  }

  fn memory_sensor_descriptors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();
    for (suffix, name, unit) in [
      ("used", "Memory Used", "bytes"),
      ("total", "Memory Total", "bytes"),
      ("utilization", "Memory Utilization", "percent"),
      ("available", "Memory Available", "bytes"),
    ] {
      if let Ok(id) = SensorId::new(format!("memory.{}", suffix)) {
        sensors.push(SensorDescriptor {
          id,
          name: name.to_string(),
          category: "memory".to_string(),
          unit: unit.to_string(),
          device: None,
          tags: vec!["memory".to_string()],
        });
      }
    }
    sensors
  }

  fn swap_sensor_descriptors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();
    for (suffix, name, unit) in [
      ("used", "Swap Used", "bytes"),
      ("total", "Swap Total", "bytes"),
      ("utilization", "Swap Utilization", "percent"),
      ("free", "Swap Free", "bytes"),
    ] {
      if let Ok(id) = SensorId::new(format!("swap.{}", suffix)) {
        sensors.push(SensorDescriptor {
          id,
          name: name.to_string(),
          category: "swap".to_string(),
          unit: unit.to_string(),
          device: None,
          tags: vec!["swap".to_string()],
        });
      }
    }
    sensors
  }

  fn disk_sensor_descriptors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
      for line in content.lines() {
        if let Some((name, _)) = DiskStats::parse_from_line(line) {
          if name.starts_with("loop") || name.starts_with("ram") {
            continue;
          }

          let safe_name = name.replace(|c: char| !c.is_ascii_alphanumeric(), "-");
          for (suffix, display_name, unit) in [
            ("read_bytes", "Read Bytes", "bytes"),
            ("write_bytes", "Write Bytes", "bytes"),
          ] {
            if let Ok(id) = SensorId::new(format!("disk.{}.{}", safe_name, suffix)) {
              sensors.push(SensorDescriptor {
                id,
                name: format!("{} {}", name, display_name),
                category: "disk".to_string(),
                unit: unit.to_string(),
                device: Some(name.clone()),
                tags: vec!["disk".to_string(), safe_name.clone()],
              });
            }
          }
        }
      }
    }

    sensors
  }

  fn network_sensor_descriptors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
      for line in content.lines().skip(2) {
        if let Some((name, _)) = NetStats::parse_from_line(line) {
          let safe_name = name.replace(|c: char| !c.is_ascii_alphanumeric(), "-");
          for (suffix, display_name, unit) in [
            ("rx_bytes", "RX Bytes", "bytes"),
            ("tx_bytes", "TX Bytes", "bytes"),
          ] {
            if let Ok(id) = SensorId::new(format!("network.{}.{}", safe_name, suffix)) {
              sensors.push(SensorDescriptor {
                id,
                name: format!("{} {}", name, display_name),
                category: "network".to_string(),
                unit: unit.to_string(),
                device: Some(name.clone()),
                tags: vec!["network".to_string(), safe_name.clone()],
              });
            }
          }
        }
      }
    }

    sensors
  }

  fn hwmon_sensor_descriptors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();
    let hwmon_path = Path::new("/sys/class/hwmon");

    if !hwmon_path.exists() {
      return sensors;
    }

    if let Ok(entries) = std::fs::read_dir(hwmon_path) {
      for entry in entries.flatten() {
        let hwmon_dir = entry.path();

        let name = std::fs::read_to_string(hwmon_dir.join("name"))
          .unwrap_or_else(|_| "unknown".to_string())
          .trim()
          .to_lowercase();

        sensors.extend(Self::find_temp_sensor_descriptors(&hwmon_dir, &name));
      }
    }

    sensors
  }

  fn find_temp_sensor_descriptors(hwmon_dir: &Path, device_name: &str) -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    if let Ok(entries) = std::fs::read_dir(hwmon_dir) {
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

        let label = std::fs::read_to_string(hwmon_dir.join(format!("temp{}_label", index)))
          .ok()
          .map(|s| s.trim().to_string());

        let sensor_id = Self::make_temp_sensor_id(device_name, &index);

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

  fn make_temp_sensor_id(device_name: &str, index: &str) -> String {
    if device_name.contains("coretemp")
      || device_name.contains("cpu")
      || device_name.contains("k10temp")
      || device_name.contains("k8temp")
    {
      format!("cpu.{}.temp{}.temperature", device_name, index)
    } else if device_name.contains("gpu") || device_name.contains("nvidia") {
      format!("gpu.{}.temp{}.temperature", device_name, index)
    } else {
      format!(
        "{}.temp{}.temperature",
        device_name.replace(' ', "_"),
        index
      )
    }
  }

  fn current_timestamp_ms() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0)
  }

  async fn read_cpu_stats() -> Option<CpuStats> {
    let content = tokio::fs::read_to_string("/proc/stat").await.ok()?;
    let first_line = content.lines().next()?;
    CpuStats::parse_from_line(first_line)
  }

  async fn read_meminfo() -> Option<HashMap<String, u64>> {
    let content = tokio::fs::read_to_string("/proc/meminfo").await.ok()?;
    let mut meminfo = HashMap::new();

    for line in content.lines() {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() >= 2 {
        let key = parts[0].trim_end_matches(':');
        if let Ok(kb) = parts[1].parse::<u64>() {
          meminfo.insert(key.to_string(), kb * 1024);
        }
      }
    }

    Some(meminfo)
  }

  async fn read_diskstats() -> Option<HashMap<String, DiskStats>> {
    let content = tokio::fs::read_to_string("/proc/diskstats").await.ok()?;
    let mut stats = HashMap::new();

    for line in content.lines() {
      if let Some((name, ds)) = DiskStats::parse_from_line(line) {
        stats.insert(name, ds);
      }
    }

    Some(stats)
  }

  async fn read_net_dev() -> Option<HashMap<String, NetStats>> {
    let content = tokio::fs::read_to_string("/proc/net/dev").await.ok()?;
    let mut stats = HashMap::new();

    for line in content.lines().skip(2) {
      if let Some((name, ns)) = NetStats::parse_from_line(line) {
        stats.insert(name, ns);
      }
    }

    Some(stats)
  }

  async fn poll_cpu(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let current_stats = match Self::read_cpu_stats().await {
      Some(stats) => stats,
      None => {
        tracing::warn!("Failed to read /proc/stat for CPU utilization");
        return;
      }
    };

    let mut prev_guard = self.prev_stats.lock().unwrap();
    if let Some(prev_stats) = prev_guard.cpu {
      if let Some(utilization) = CpuStats::utilization_from(prev_stats, current_stats) {
        if let Ok(id) = SensorId::new("cpu.utilization") {
          samples.push(SensorSample {
            sensor_id: id,
            timestamp_ms,
            value: Some(utilization),
          });
        }
      }
    }

    prev_guard.cpu = Some(current_stats);
  }

  async fn poll_memory(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let meminfo = match Self::read_meminfo().await {
      Some(info) => info,
      None => {
        tracing::warn!("Failed to read /proc/meminfo");
        return;
      }
    };

    let mem_total = meminfo.get("MemTotal").copied().unwrap_or(0);
    let mem_available = meminfo.get("MemAvailable").copied().unwrap_or(0);
    let mem_used = mem_total.saturating_sub(mem_available);
    let mem_utilization = if mem_total > 0 {
      Some((mem_used as f64 / mem_total as f64) * 100.0)
    } else {
      None
    };

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
  }

  async fn poll_swap(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let meminfo = match Self::read_meminfo().await {
      Some(info) => info,
      None => return,
    };

    let swap_total = meminfo.get("SwapTotal").copied().unwrap_or(0);
    let swap_free = meminfo.get("SwapFree").copied().unwrap_or(0);
    let swap_used = swap_total.saturating_sub(swap_free);
    let swap_utilization = if swap_total > 0 {
      Some((swap_used as f64 / swap_total as f64) * 100.0)
    } else {
      None
    };

    if let Ok(id) = SensorId::new("swap.total") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: Some(swap_total as f64),
      });
    }
    if let Ok(id) = SensorId::new("swap.used") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: Some(swap_used as f64),
      });
    }
    if let Ok(id) = SensorId::new("swap.free") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: Some(swap_free as f64),
      });
    }
    if let Ok(id) = SensorId::new("swap.utilization") {
      samples.push(SensorSample {
        sensor_id: id,
        timestamp_ms,
        value: swap_utilization,
      });
    }
  }

  async fn poll_disk(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let current_stats = match Self::read_diskstats().await {
      Some(stats) => stats,
      None => {
        tracing::trace!("Failed to read /proc/diskstats");
        return;
      }
    };

    let mut prev_guard = self.prev_stats.lock().unwrap();

    for (name, curr) in &current_stats {
      if name.starts_with("loop") || name.starts_with("ram") {
        continue;
      }

      let safe_name = name.replace(|c: char| !c.is_ascii_alphanumeric(), "-");

      if let Some(prev) = prev_guard.disks.get(name) {
        let read_delta = DiskStats::read_bytes_delta(*prev, *curr);
        let write_delta = DiskStats::write_bytes_delta(*prev, *curr);

        if let Ok(id) = SensorId::new(format!("disk.{}.read_bytes", safe_name)) {
          samples.push(SensorSample {
            sensor_id: id,
            timestamp_ms,
            value: Some(read_delta as f64),
          });
        }
        if let Ok(id) = SensorId::new(format!("disk.{}.write_bytes", safe_name)) {
          samples.push(SensorSample {
            sensor_id: id,
            timestamp_ms,
            value: Some(write_delta as f64),
          });
        }
      }
    }

    prev_guard.disks = current_stats;
  }

  async fn poll_network(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let current_stats = match Self::read_net_dev().await {
      Some(stats) => stats,
      None => {
        tracing::trace!("Failed to read /proc/net/dev");
        return;
      }
    };

    let mut prev_guard = self.prev_stats.lock().unwrap();

    for (name, curr) in &current_stats {
      let safe_name = name.replace(|c: char| !c.is_ascii_alphanumeric(), "-");

      if let Some(prev) = prev_guard.networks.get(name) {
        let rx_delta = NetStats::rx_delta(*prev, *curr);
        let tx_delta = NetStats::tx_delta(*prev, *curr);

        if let Ok(id) = SensorId::new(format!("network.{}.rx_bytes", safe_name)) {
          samples.push(SensorSample {
            sensor_id: id,
            timestamp_ms,
            value: Some(rx_delta as f64),
          });
        }
        if let Ok(id) = SensorId::new(format!("network.{}.tx_bytes", safe_name)) {
          samples.push(SensorSample {
            sensor_id: id,
            timestamp_ms,
            value: Some(tx_delta as f64),
          });
        }
      }
    }

    prev_guard.networks = current_stats;
  }

  async fn poll_temperatures(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let hwmon_path = Path::new("/sys/class/hwmon");

    if !hwmon_path.exists() {
      return;
    }

    let entries = match tokio::fs::read_dir(hwmon_path).await {
      Ok(e) => e,
      Err(e) => {
        tracing::warn!("Failed to read hwmon directory: {}", e);
        return;
      }
    };

    let mut entries = entries;
    while let Ok(Some(entry)) = entries.next_entry().await {
      let hwmon_dir = entry.path();

      let device_name = match tokio::fs::read_to_string(hwmon_dir.join("name")).await {
        Ok(name) => name.trim().to_lowercase(),
        Err(_) => continue,
      };

      if let Ok(mut dir_entries) = tokio::fs::read_dir(&hwmon_dir).await {
        while let Ok(Some(file_entry)) = dir_entries.next_entry().await {
          let file_name = file_entry.file_name();
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

          let temp_content = match tokio::fs::read_to_string(file_entry.path()).await {
            Ok(c) => c,
            Err(_) => continue,
          };

          let temp_mc: i64 = match temp_content.trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
          };

          let temp_c = temp_mc as f64 / 1000.0;

          let sensor_id_str = Self::make_temp_sensor_id(&device_name, &index);
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
        "swap".to_string(),
        "disk".to_string(),
        "network".to_string(),
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

  fn poll_interval(&self) -> Duration {
    self.poll_interval
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    Ok(self.sensors.clone())
  }

  async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
    let mut samples = Vec::new();
    let timestamp_ms = Self::current_timestamp_ms();

    self.poll_cpu(&mut samples, timestamp_ms).await;
    self.poll_memory(&mut samples, timestamp_ms).await;
    self.poll_swap(&mut samples, timestamp_ms).await;
    self.poll_disk(&mut samples, timestamp_ms).await;
    self.poll_network(&mut samples, timestamp_ms).await;
    self.poll_temperatures(&mut samples, timestamp_ms).await;

    {
      let mut prev = self.prev_stats.lock().unwrap();
      prev.timestamp_ms = timestamp_ms;
    }

    tracing::trace!("Linux provider polled {} samples", samples.len());
    Ok(samples)
  }

  async fn health(&self) -> ProviderHealth {
    let mut failures = Vec::new();

    if tokio::fs::read_to_string("/proc/stat").await.is_err() {
      failures.push("/proc/stat unreadable");
    }
    if tokio::fs::read_to_string("/proc/meminfo").await.is_err() {
      failures.push("/proc/meminfo unreadable");
    }

    if failures.is_empty() {
      ProviderHealth::Ok
    } else {
      ProviderHealth::Degraded {
        message: failures.join(", "),
      }
    }
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
      user: 150,
      nice: 0,
      system: 75,
      idle: 900,
      iowait: 15,
      irq: 7,
      softirq: 6,
      steal: 0,
      guest: 0,
      guest_nice: 0,
    };

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

    let curr = CpuStats {
      user: 200,
      idle: 100,
      ..CpuStats::default()
    };

    let utilization = CpuStats::utilization_from(prev, curr);
    assert!(utilization.is_some());
    assert!((utilization.unwrap() - 100.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_cpu_stats_parse_valid() {
    let line = "cpu  100 0 50 800 10 5 5 0 0 0";
    let stats = CpuStats::parse_from_line(line).unwrap();
    assert_eq!(stats.user, 100);
    assert_eq!(stats.system, 50);
    assert_eq!(stats.idle, 800);
  }

  #[test]
  fn test_cpu_stats_parse_invalid_prefix() {
    let line = "cpu0 100 0 50 800 10 5 5 0 0 0";
    assert!(CpuStats::parse_from_line(line).is_none());
  }

  #[test]
  fn test_disk_stats_parse() {
    let line = "  8       0 sda 100 50 2000 100 200 100 4000 200";
    let (name, stats) = DiskStats::parse_from_line(line).unwrap();
    assert_eq!(name, "sda");
    assert_eq!(stats.sectors_read, 2000);
    assert_eq!(stats.sectors_written, 4000);
  }

  #[test]
  fn test_disk_stats_delta() {
    let prev = DiskStats {
      sectors_read: 2000,
      sectors_written: 4000,
      ..DiskStats::default()
    };
    let curr = DiskStats {
      sectors_read: 3000,
      sectors_written: 6000,
      ..DiskStats::default()
    };
    assert_eq!(DiskStats::read_bytes_delta(prev, curr), 1000 * 512);
    assert_eq!(DiskStats::write_bytes_delta(prev, curr), 2000 * 512);
  }

  #[test]
  fn test_net_stats_parse() {
    let line = "  eth0: 1000    0    0    0    0     0          0        0 2000    0    0    0    0     0          0        0";
    let (name, stats) = NetStats::parse_from_line(line).unwrap();
    assert_eq!(name, "eth0");
    assert_eq!(stats.rx_bytes, 1000);
    assert_eq!(stats.tx_bytes, 2000);
  }

  #[test]
  fn test_net_stats_skip_loopback() {
    let line = "  lo: 100    0    0    0    0     0          0        0 100    0    0    0    0     0          0        0";
    assert!(NetStats::parse_from_line(line).is_none());
  }

  #[test]
  fn test_net_stats_delta() {
    let prev = NetStats {
      rx_bytes: 1000,
      tx_bytes: 2000,
    };
    let curr = NetStats {
      rx_bytes: 3000,
      tx_bytes: 5000,
    };
    assert_eq!(NetStats::rx_delta(prev, curr), 2000);
    assert_eq!(NetStats::tx_delta(prev, curr), 3000);
  }

  #[test]
  fn test_current_timestamp_ms() {
    let ts = LinuxProvider::current_timestamp_ms();
    assert!(ts > 1577836800000);
  }

  #[test]
  fn test_sensor_descriptors_valid_ids() {
    let sensors = LinuxProvider::discover_sensors_sync();
    for sensor in &sensors {
      assert!(
        !sensor.name.is_empty(),
        "Sensor {} missing name",
        sensor.id.as_str()
      );
      assert!(
        !sensor.category.is_empty(),
        "Sensor {} missing category",
        sensor.id.as_str()
      );
      assert!(
        !sensor.unit.is_empty(),
        "Sensor {} missing unit",
        sensor.id.as_str()
      );
    }
  }

  #[test]
  fn test_make_temp_sensor_id_cpu() {
    assert_eq!(
      LinuxProvider::make_temp_sensor_id("coretemp", "1"),
      "cpu.coretemp.temp1.temperature"
    );
    assert_eq!(
      LinuxProvider::make_temp_sensor_id("k10temp", "2"),
      "cpu.k10temp.temp2.temperature"
    );
  }

  #[test]
  fn test_make_temp_sensor_id_gpu() {
    assert_eq!(
      LinuxProvider::make_temp_sensor_id("nvidia", "1"),
      "gpu.nvidia.temp1.temperature"
    );
  }

  #[test]
  fn test_make_temp_sensor_id_other() {
    assert_eq!(
      LinuxProvider::make_temp_sensor_id("acpitz", "1"),
      "acpitz.temp1.temperature"
    );
  }
}
