//! Linux system provider - reads from /proc and /sys for CPU, GPU, memory, and
//! temperature sensors.
//!
//! Emits the 7 canonical sensor ids the default instrument panel binds to:
//!   cpu.total.utilization, cpu.clock, cpu.temperature,
//!   gpu.total.utilization, gpu.temperature,
//!   memory.used.percent, memory.used
//! plus a handful of best-effort extras (memory.total, memory.available, and
//! granular per-hwmon temperatures). Unit strings match the MockProvider demo
//! exactly so the LinuxProvider is interchangeable with the mock.
//!
//! Reads only /proc and /sys (no external binaries). A missing sensor source is
//! skipped and logged, never panicked on.

#[cfg(target_os = "linux")]
use async_trait::async_trait;
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};
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

// ---------------------------------------------------------------------------
// Pure parsing / selection helpers (no file IO — unit-testable on any OS / CI).
// These are NOT gated on target_os so the CI tests on ubuntu (without
// amdgpu/k10temp) exercise them without ever touching real /sys.
// ---------------------------------------------------------------------------

/// Average a set of `scaling_cur_freq` readings (kHz, one per line/string) and
/// convert to MHz. Returns None if no value parses.
pub(crate) fn mean_freq_mhz_from_khz<S: AsRef<str>>(samples: &[S]) -> Option<f64> {
  let mut sum_khz: f64 = 0.0;
  let mut count: u64 = 0;
  for s in samples {
    if let Ok(khz) = s.as_ref().trim().parse::<f64>() {
      sum_khz += khz;
      count += 1;
    }
  }
  if count == 0 {
    return None;
  }
  Some((sum_khz / count as f64) / 1000.0)
}

/// Average the "cpu MHz" lines of /proc/cpuinfo (already in MHz). Fallback for
/// machines without cpufreq. Returns None if no value parses.
pub(crate) fn mean_cpuinfo_mhz(cpuinfo: &str) -> Option<f64> {
  let mut sum: f64 = 0.0;
  let mut count: u64 = 0;
  for line in cpuinfo.lines() {
    let lower = line.to_ascii_lowercase();
    if lower.starts_with("cpu mhz") {
      if let Some((_, val)) = line.split_once(':') {
        if let Ok(mhz) = val.trim().parse::<f64>() {
          sum += mhz;
          count += 1;
        }
      }
    }
  }
  if count == 0 {
    return None;
  }
  Some(sum / count as f64)
}

/// Convert a millidegree-Celsius reading (e.g. "50125") to Celsius.
pub(crate) fn millidegrees_to_celsius(raw: &str) -> Option<f64> {
  raw.trim().parse::<i64>().ok().map(|mc| mc as f64 / 1000.0)
}

/// A discrete-GPU candidate: a drm card with readable gpu_busy_percent AND
/// mem_info_vram_total. `key` is anything that identifies the card to the
/// caller (e.g. its device path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GpuCandidate<K> {
  pub key: K,
  pub vram_total: u64,
}

/// Pick the discrete GPU: the candidate with the LARGEST mem_info_vram_total.
/// Ties resolve to the first seen. Returns None if there are no candidates.
pub(crate) fn pick_discrete_gpu<K: Clone>(candidates: &[GpuCandidate<K>]) -> Option<K> {
  candidates
    .iter()
    .max_by_key(|c| c.vram_total)
    .map(|c| c.key.clone())
}

/// Compute used-memory MB and used-percent from a meminfo map of kB values.
/// Returns `(used_mb, used_percent)`; either may be None if inputs are missing.
pub(crate) fn meminfo_used(meminfo: &HashMap<String, u64>) -> (Option<f64>, Option<f64>) {
  let total_kb = match meminfo.get("MemTotal").copied() {
    Some(v) if v > 0 => v,
    _ => return (None, None),
  };
  let avail_kb = meminfo.get("MemAvailable").copied().unwrap_or(0);
  let used_kb = total_kb.saturating_sub(avail_kb);

  // kB -> MB: /1024 (MemInfo is kB; bytes would be *1024, MB is /1024/1024).
  let used_mb = used_kb as f64 / 1024.0;
  let used_percent = (used_kb as f64 / total_kb as f64) * 100.0;
  (Some(used_mb), Some(used_percent))
}

// ---------------------------------------------------------------------------
// CPU utilization stats (/proc/stat).
// ---------------------------------------------------------------------------

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
  /// device path of the discrete GPU, chosen once at construction so discover()
  /// and poll() stay in lockstep. None if no discrete GPU candidate exists.
  discrete_gpu: Option<PathBuf>,
}

#[cfg(target_os = "linux")]
impl LinuxProvider {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    tracing::info!("Initializing Linux provider");
    let discrete_gpu = Self::find_discrete_gpu();
    match &discrete_gpu {
      Some(p) => tracing::info!("Selected discrete GPU: {}", p.display()),
      None => tracing::info!("No discrete GPU detected; gpu.* sensors omitted"),
    }
    let sensors = Self::discover_sensors(discrete_gpu.as_deref());
    tracing::info!("Linux provider discovered {} sensors", sensors.len());
    Self {
      manifest: linux_manifest(),
      sensors,
      prev_cpu_stats: Arc::new(Mutex::new(None)),
      discrete_gpu,
    }
  }

  fn discover_sensors(discrete_gpu: Option<&Path>) -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    sensors.extend(Self::discover_cpu_sensors());
    sensors.extend(Self::discover_memory_sensors());
    sensors.extend(Self::discover_temperature_sensors());
    sensors.extend(Self::discover_gpu_sensors(discrete_gpu));

    sensors
  }

  // --- CPU descriptors --------------------------------------------------------

  fn discover_cpu_sensors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();

    // cpu.total.utilization is always discoverable (/proc/stat always exists).
    push_descriptor(
      &mut sensors,
      "cpu.total.utilization",
      "CPU Utilization",
      "utilization",
      "%",
      None,
      &["cpu"],
    );

    // cpu.clock — only if we can read a frequency from cpufreq or cpuinfo.
    if Self::read_cpu_clock_mhz().is_some() {
      push_descriptor(
        &mut sensors,
        "cpu.clock",
        "CPU Clock",
        "frequency",
        "MHz",
        None,
        &["cpu"],
      );
    }

    // cpu.temperature — only if we can locate a CPU package temperature.
    if Self::read_cpu_temperature_c().is_some() {
      push_descriptor(
        &mut sensors,
        "cpu.temperature",
        "CPU Temperature",
        "temperature",
        "°C",
        None,
        &["cpu", "thermal"],
      );
    }

    sensors
  }

  // --- Memory descriptors -----------------------------------------------------

  fn discover_memory_sensors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();
    if fs::read_to_string("/proc/meminfo").is_err() {
      tracing::warn!("Failed to read /proc/meminfo; memory sensors omitted");
      return sensors;
    }

    // Canonical.
    push_descriptor(
      &mut sensors,
      "memory.used.percent",
      "Memory Used",
      "utilization",
      "%",
      None,
      &["memory"],
    );
    push_descriptor(
      &mut sensors,
      "memory.used",
      "Memory Used",
      "memory",
      "MB",
      None,
      &["memory"],
    );

    // Extras (best-effort), aligned to MB for consistency with memory.used.
    push_descriptor(
      &mut sensors,
      "memory.total",
      "Memory Total",
      "memory",
      "MB",
      None,
      &["memory"],
    );
    push_descriptor(
      &mut sensors,
      "memory.available",
      "Memory Available",
      "memory",
      "MB",
      None,
      &["memory"],
    );

    sensors
  }

  // --- Temperature descriptors (canonical cpu.temperature handled above) ------

  /// Discover granular per-hwmon temperature sensors as best-effort extras.
  /// The canonical cpu.temperature / gpu.temperature are emitted elsewhere; the
  /// extras here use device-qualified ids and never collide with the canonical 7.
  fn discover_temperature_sensors() -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();
    let hwmon_path = Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() {
      return sensors;
    }

    let entries = match fs::read_dir(hwmon_path) {
      Ok(e) => e,
      Err(e) => {
        tracing::warn!("Failed to read {}: {}", hwmon_path.display(), e);
        return sensors;
      }
    };

    for entry in entries.flatten() {
      let hwmon_dir = entry.path();
      let device_name = fs::read_to_string(hwmon_dir.join("name"))
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
      if device_name.is_empty() {
        continue;
      }

      for (index, _label) in Self::temp_inputs(&hwmon_dir) {
        let id = make_temp_sensor_id(&device_name, &index);
        // Only valid (and non-canonical) ids survive; collisions/invalid ids are skipped.
        if id == "cpu.temperature" || id == "gpu.temperature" {
          continue;
        }
        let label = fs::read_to_string(hwmon_dir.join(format!("temp{}_label", index)))
          .ok()
          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty());
        let display = label.unwrap_or_else(|| format!("{} temp{}", device_name, index));
        push_descriptor_owned(
          &mut sensors,
          id,
          display,
          "temperature".to_string(),
          "°C".to_string(),
          Some(device_name.clone()),
          vec!["thermal".to_string(), device_name.clone()],
        );
      }
    }

    sensors
  }

  // --- GPU descriptors --------------------------------------------------------

  fn discover_gpu_sensors(discrete_gpu: Option<&Path>) -> Vec<SensorDescriptor> {
    let mut sensors = Vec::new();
    let Some(card) = discrete_gpu else {
      return sensors;
    };

    if Self::read_gpu_busy_percent(card).is_some() {
      push_descriptor(
        &mut sensors,
        "gpu.total.utilization",
        "GPU Utilization",
        "utilization",
        "%",
        Some("gpu".to_string()),
        &["gpu"],
      );
    }
    if Self::read_gpu_temperature_c(card).is_some() {
      push_descriptor(
        &mut sensors,
        "gpu.temperature",
        "GPU Temperature",
        "temperature",
        "°C",
        Some("gpu".to_string()),
        &["gpu", "thermal"],
      );
    }

    sensors
  }

  // ------------------------------------------------------------------------
  // Discrete-GPU selection (deterministic): iterate /sys/class/drm/card*/device,
  // candidate = readable gpu_busy_percent AND mem_info_vram_total, pick largest
  // vram. Connector dirs (card1-DP-1) lack both files and are filtered out.
  // ------------------------------------------------------------------------
  fn find_discrete_gpu() -> Option<PathBuf> {
    let drm = Path::new("/sys/class/drm");
    let entries = fs::read_dir(drm).ok()?;

    let mut candidates: Vec<GpuCandidate<PathBuf>> = Vec::new();
    for entry in entries.flatten() {
      let device = entry.path().join("device");
      let busy = device.join("gpu_busy_percent");
      let vram = device.join("mem_info_vram_total");

      let busy_ok = fs::read_to_string(&busy)
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .is_some();
      let vram_total = fs::read_to_string(&vram)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());

      if let (true, Some(vram_total)) = (busy_ok, vram_total) {
        candidates.push(GpuCandidate {
          key: device,
          vram_total,
        });
      }
    }

    pick_discrete_gpu(&candidates)
  }

  // ------------------------------------------------------------------------
  // Raw readers (file IO; delegate math to the pure helpers above).
  // ------------------------------------------------------------------------

  /// Get current timestamp in milliseconds since UNIX epoch.
  fn current_timestamp_ms() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0)
  }

  /// Parse /proc/stat first line (aggregate CPU stats).
  fn read_cpu_stats() -> Option<CpuStats> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let first_line = content.lines().next()?;

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

  /// Mean CPU clock in MHz: average of every readable
  /// /sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq (kHz -> MHz).
  /// Fallback: mean of "cpu MHz" lines in /proc/cpuinfo.
  fn read_cpu_clock_mhz() -> Option<f64> {
    let mut freqs: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
      for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Match cpu0, cpu1, ... but not cpufreq, cpuidle, etc.
        if !(name.starts_with("cpu")
          && name.len() > 3
          && name[3..].chars().all(|c| c.is_ascii_digit()))
        {
          continue;
        }
        let path = entry.path().join("cpufreq/scaling_cur_freq");
        if let Ok(content) = fs::read_to_string(&path) {
          freqs.push(content);
        }
      }
    }

    if let Some(mhz) = mean_freq_mhz_from_khz(&freqs) {
      return Some(mhz);
    }

    // Fallback to /proc/cpuinfo.
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    mean_cpuinfo_mhz(&cpuinfo)
  }

  /// CPU package temperature in °C. Prefers k10temp "Tctl" (AMD) or coretemp
  /// "Package id 0" (Intel); falls back to the first temp of
  /// k10temp/coretemp/zenpower.
  fn read_cpu_temperature_c() -> Option<f64> {
    let hwmon_path = Path::new("/sys/class/hwmon");
    let entries = fs::read_dir(hwmon_path).ok()?;

    let mut fallback: Option<f64> = None;

    for entry in entries.flatten() {
      let dir = entry.path();
      let device_name = fs::read_to_string(dir.join("name"))
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();

      let is_cpu = device_name == "k10temp"
        || device_name == "coretemp"
        || device_name == "zenpower"
        || device_name == "k8temp";
      if !is_cpu {
        continue;
      }

      for (index, label) in Self::temp_inputs(&dir) {
        let label_lc = label.to_ascii_lowercase();
        let preferred = label_lc == "tctl" || label_lc == "package id 0";

        if let Some(c) = Self::read_temp_input_c(&dir, &index) {
          if preferred {
            return Some(c);
          }
          if fallback.is_none() {
            fallback = Some(c);
          }
        }
      }
    }

    fallback
  }

  /// Read the discrete GPU's busy percent from card/device/gpu_busy_percent.
  fn read_gpu_busy_percent(card_device: &Path) -> Option<f64> {
    fs::read_to_string(card_device.join("gpu_busy_percent"))
      .ok()?
      .trim()
      .parse::<f64>()
      .ok()
      .map(|v| v.clamp(0.0, 100.0))
  }

  /// Read the discrete GPU's temperature in °C from the SAME card's hwmon:
  /// card/device/hwmon/hwmon*/tempK_input where label=="edge", else temp1.
  fn read_gpu_temperature_c(card_device: &Path) -> Option<f64> {
    let hwmon_root = card_device.join("hwmon");
    let entries = fs::read_dir(&hwmon_root).ok()?;

    for entry in entries.flatten() {
      let dir = entry.path();
      let inputs = Self::temp_inputs(&dir);
      if inputs.is_empty() {
        continue;
      }

      // Prefer the "edge" label.
      for (index, label) in &inputs {
        if label.eq_ignore_ascii_case("edge") {
          if let Some(c) = Self::read_temp_input_c(&dir, index) {
            return Some(c);
          }
        }
      }
      // Fallback: temp1.
      if let Some(c) = Self::read_temp_input_c(&dir, "1") {
        return Some(c);
      }
    }

    None
  }

  /// Enumerate (index, label) for every tempN_input in a hwmon dir. Label is ""
  /// when no tempN_label file exists.
  fn temp_inputs(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
      return out;
    };
    for entry in entries.flatten() {
      let file_name = entry.file_name();
      let name = file_name.to_string_lossy();
      if !(name.starts_with("temp") && name.ends_with("_input")) {
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
      let label = fs::read_to_string(dir.join(format!("temp{}_label", index)))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
      out.push((index, label));
    }
    out
  }

  /// Read tempN_input from a hwmon dir and convert millidegrees -> °C.
  fn read_temp_input_c(dir: &Path, index: &str) -> Option<f64> {
    let raw = fs::read_to_string(dir.join(format!("temp{}_input", index))).ok()?;
    millidegrees_to_celsius(&raw)
  }

  /// Parse /proc/meminfo into a map of kB values (raw kB, not bytes).
  fn read_meminfo_kb() -> Option<HashMap<String, u64>> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut meminfo = HashMap::new();
    for line in content.lines() {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() >= 2 {
        let key = parts[0].trim_end_matches(':');
        if let Ok(kb) = parts[1].parse::<u64>() {
          meminfo.insert(key.to_string(), kb);
        }
      }
    }
    Some(meminfo)
  }

  // ------------------------------------------------------------------------
  // Pollers.
  // ------------------------------------------------------------------------

  fn poll_cpu(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    // Utilization (requires delta from a previous poll).
    match Self::read_cpu_stats() {
      Some(current_stats) => {
        let mut prev_guard = self.prev_cpu_stats.lock().unwrap();
        if let Some(prev_stats) = prev_guard.as_ref() {
          if let Some(util) = CpuStats::utilization_from(*prev_stats, current_stats) {
            push_sample(samples, "cpu.total.utilization", timestamp_ms, Some(util));
          }
        }
        *prev_guard = Some(current_stats);
      }
      None => tracing::warn!("Failed to read /proc/stat for CPU utilization"),
    }

    // Clock.
    if let Some(mhz) = Self::read_cpu_clock_mhz() {
      push_sample(samples, "cpu.clock", timestamp_ms, Some(mhz));
    }

    // Temperature.
    if let Some(c) = Self::read_cpu_temperature_c() {
      push_sample(samples, "cpu.temperature", timestamp_ms, Some(c));
    }
  }

  fn poll_memory(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let meminfo = match Self::read_meminfo_kb() {
      Some(info) => info,
      None => {
        tracing::warn!("Failed to read /proc/meminfo");
        return;
      }
    };

    let (used_mb, used_percent) = meminfo_used(&meminfo);
    push_sample(samples, "memory.used.percent", timestamp_ms, used_percent);
    push_sample(samples, "memory.used", timestamp_ms, used_mb);

    // Extras in MB.
    if let Some(total_kb) = meminfo.get("MemTotal").copied() {
      push_sample(
        samples,
        "memory.total",
        timestamp_ms,
        Some(total_kb as f64 / 1024.0),
      );
    }
    if let Some(avail_kb) = meminfo.get("MemAvailable").copied() {
      push_sample(
        samples,
        "memory.available",
        timestamp_ms,
        Some(avail_kb as f64 / 1024.0),
      );
    }
  }

  fn poll_temperatures(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    // Canonical cpu.temperature is emitted by poll_cpu. Here we emit only the
    // granular per-hwmon extras (skipping any that would collide with canonical).
    let hwmon_path = Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() {
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
      let dir = entry.path();
      let device_name = fs::read_to_string(dir.join("name"))
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
      if device_name.is_empty() {
        continue;
      }

      for (index, _label) in Self::temp_inputs(&dir) {
        let id = make_temp_sensor_id(&device_name, &index);
        if id == "cpu.temperature" || id == "gpu.temperature" {
          continue;
        }
        if let Some(c) = Self::read_temp_input_c(&dir, &index) {
          push_sample_owned(samples, id, timestamp_ms, Some(c));
        }
      }
    }
  }

  fn poll_gpu(&self, samples: &mut Vec<SensorSample>, timestamp_ms: u64) {
    let Some(card) = self.discrete_gpu.as_deref() else {
      return;
    };
    if let Some(busy) = Self::read_gpu_busy_percent(card) {
      push_sample(samples, "gpu.total.utilization", timestamp_ms, Some(busy));
    }
    if let Some(c) = Self::read_gpu_temperature_c(card) {
      push_sample(samples, "gpu.temperature", timestamp_ms, Some(c));
    }
  }
}

// ---------------------------------------------------------------------------
// Small free helpers shared by discover/poll (keep IDs / units in one place).
// ---------------------------------------------------------------------------

/// Build a device-qualified id for a granular per-hwmon temp extra.
/// Returns an empty string for names that cannot form a valid id; the caller
/// then skips it via SensorId::new failing. Canonical ids are produced
/// elsewhere — this is for extras only.
#[cfg(target_os = "linux")]
fn make_temp_sensor_id(device_name: &str, index: &str) -> String {
  // Sanitize: SensorId allows only [a-z0-9.]. Replace anything else with nothing
  // so we don't emit invalid ids (e.g. "r8169_0_800:00" -> "r8169080000").
  let sanitized: String = device_name
    .chars()
    .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    .collect();
  if sanitized.is_empty() {
    return String::new();
  }
  if sanitized.contains("coretemp")
    || sanitized.contains("k10temp")
    || sanitized.contains("k8temp")
    || sanitized.contains("zenpower")
  {
    format!("cpu.{}.temp{}.temperature", sanitized, index)
  } else if sanitized.contains("amdgpu") || sanitized.contains("nvidia") {
    format!("gpu.{}.temp{}.temperature", sanitized, index)
  } else {
    format!("{}.temp{}.temperature", sanitized, index)
  }
}

#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
fn push_descriptor(
  out: &mut Vec<SensorDescriptor>,
  id: &str,
  name: &str,
  category: &str,
  unit: &str,
  device: Option<String>,
  tags: &[&str],
) {
  push_descriptor_owned(
    out,
    id.to_string(),
    name.to_string(),
    category.to_string(),
    unit.to_string(),
    device,
    tags.iter().map(|s| s.to_string()).collect(),
  );
}

#[cfg(target_os = "linux")]
fn push_descriptor_owned(
  out: &mut Vec<SensorDescriptor>,
  id: String,
  name: String,
  category: String,
  unit: String,
  device: Option<String>,
  tags: Vec<String>,
) {
  match SensorId::new(&id) {
    Ok(sensor_id) => out.push(SensorDescriptor {
      id: sensor_id,
      name,
      category,
      unit,
      device,
      tags,
    }),
    Err(e) => tracing::debug!("Skipping invalid sensor id '{}': {}", id, e),
  }
}

#[cfg(target_os = "linux")]
fn push_sample(out: &mut Vec<SensorSample>, id: &str, timestamp_ms: u64, value: Option<f64>) {
  push_sample_owned(out, id.to_string(), timestamp_ms, value);
}

#[cfg(target_os = "linux")]
fn push_sample_owned(
  out: &mut Vec<SensorSample>,
  id: String,
  timestamp_ms: u64,
  value: Option<f64>,
) {
  match SensorId::new(&id) {
    Ok(sensor_id) => out.push(SensorSample {
      sensor_id,
      timestamp_ms,
      value,
    }),
    Err(e) => tracing::debug!("Skipping invalid sample id '{}': {}", id, e),
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
        "utilization".to_string(),
        "frequency".to_string(),
        "temperature".to_string(),
        "memory".to_string(),
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

    self.poll_cpu(&mut samples, timestamp_ms);
    self.poll_memory(&mut samples, timestamp_ms);
    self.poll_temperatures(&mut samples, timestamp_ms);
    self.poll_gpu(&mut samples, timestamp_ms);

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mean_freq_averages_and_converts_khz_to_mhz() {
    let samples = ["5479613\n", "5517802\n", "5506199"];
    let mhz = mean_freq_mhz_from_khz(&samples).expect("should compute");
    // mean kHz = 5501204.67 -> ~5501.2 MHz
    assert!((mhz - 5501.204).abs() < 0.5, "got {}", mhz);
  }

  #[test]
  fn mean_freq_empty_is_none() {
    let empty: [&str; 0] = [];
    assert!(mean_freq_mhz_from_khz(&empty).is_none());
    let garbage = ["abc", ""];
    assert!(mean_freq_mhz_from_khz(&garbage).is_none());
  }

  #[test]
  fn cpuinfo_mhz_fallback_averages() {
    let cpuinfo = "processor\t: 0\ncpu MHz\t\t: 4000.000\nprocessor\t: 1\ncpu MHz\t\t: 4200.000\n";
    let mhz = mean_cpuinfo_mhz(cpuinfo).expect("should compute");
    assert!((mhz - 4100.0).abs() < f64::EPSILON, "got {}", mhz);
  }

  #[test]
  fn millidegrees_converts() {
    assert!((millidegrees_to_celsius("50125").unwrap() - 50.125).abs() < f64::EPSILON);
    assert!((millidegrees_to_celsius("  48000\n").unwrap() - 48.0).abs() < f64::EPSILON);
    assert!(millidegrees_to_celsius("notanumber").is_none());
  }

  #[test]
  fn pick_discrete_gpu_chooses_largest_vram() {
    let candidates = vec![
      GpuCandidate {
        key: "card0",
        vram_total: 2_147_483_648,
      },
      GpuCandidate {
        key: "card1",
        vram_total: 17_095_983_104,
      },
    ];
    assert_eq!(pick_discrete_gpu(&candidates), Some("card1"));
  }

  #[test]
  fn pick_discrete_gpu_none_when_empty() {
    let candidates: Vec<GpuCandidate<&str>> = Vec::new();
    assert!(pick_discrete_gpu(&candidates).is_none());
  }

  #[test]
  fn meminfo_used_computes_mb_and_percent() {
    let mut m = HashMap::new();
    m.insert("MemTotal".to_string(), 63_307_136u64); // kB
    m.insert("MemAvailable".to_string(), 21_401_844u64); // kB
    let (used_mb, used_pct) = meminfo_used(&m);
    // used kB = 41_905_292 -> /1024 = 40923.14 MB
    assert!(
      (used_mb.unwrap() - 40923.14).abs() < 1.0,
      "mb={:?}",
      used_mb
    );
    // percent = 41_905_292 / 63_307_136 * 100 = 66.19%
    assert!(
      (used_pct.unwrap() - 66.19).abs() < 0.5,
      "pct={:?}",
      used_pct
    );
  }

  #[test]
  fn meminfo_used_missing_total_is_none() {
    let m = HashMap::new();
    let (used_mb, used_pct) = meminfo_used(&m);
    assert!(used_mb.is_none());
    assert!(used_pct.is_none());
  }

  #[test]
  fn make_temp_id_classifies_and_sanitizes() {
    assert_eq!(
      make_temp_sensor_id("k10temp", "1"),
      "cpu.k10temp.temp1.temperature"
    );
    assert_eq!(
      make_temp_sensor_id("amdgpu", "1"),
      "gpu.amdgpu.temp1.temperature"
    );
    // Non-alphanumerics stripped so the id stays valid for SensorId.
    let id = make_temp_sensor_id("r8169_0_800:00", "1");
    assert!(
      id.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.'),
      "id had invalid chars: {}",
      id
    );
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn cpu_stats_default() {
    let stats = CpuStats::default();
    assert_eq!(stats.user, 0);
    assert_eq!(stats.idle, 0);
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn cpu_utilization_basic() {
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
    let util = CpuStats::utilization_from(prev, curr).expect("some");
    assert!(util > 42.0 && util < 43.0, "Expected ~42.6%, got {}", util);
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn cpu_utilization_zero_delta_is_none() {
    let stats = CpuStats {
      user: 100,
      idle: 800,
      ..CpuStats::default()
    };
    assert!(CpuStats::utilization_from(stats, stats).is_none());
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn cpu_utilization_clamps_to_100() {
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
    let util = CpuStats::utilization_from(prev, curr).expect("some");
    assert!((util - 100.0).abs() < f64::EPSILON);
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn timestamp_is_recent() {
    let ts = LinuxProvider::current_timestamp_ms();
    assert!(ts > 1_577_836_800_000);
  }
}
