//! Linux system provider - reads from /proc, /sys/class/hwmon for CPU, memory, and temperature sensors.

#[cfg(target_os = "linux")]
use async_trait::async_trait;
#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
use astragauge_domain::{
  ProviderCapabilities, ProviderManifest, SensorCategories, SensorDescriptor, SensorId,
  SensorSample,
};
#[cfg(target_os = "linux")]
use astragauge_provider_host::{Provider, ProviderHealth, ProviderResult};

#[cfg(target_os = "linux")]
pub struct LinuxProvider {
  manifest: ProviderManifest,
  sensors: Vec<SensorDescriptor>,
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
          "cpu.temperature".to_string()
        } else if device_name.contains("gpu") || device_name.contains("nvidia") {
          "gpu.temperature".to_string()
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

  fn poll_interval(&self) -> Duration {
    Duration::from_millis(1000)
  }

  async fn discover(&self) -> ProviderResult<Vec<SensorDescriptor>> {
    Ok(self.sensors.clone())
  }

  async fn poll(&self) -> ProviderResult<Vec<SensorSample>> {
    Ok(vec![])
  }

  async fn health(&self) -> ProviderHealth {
    ProviderHealth::Ok
  }

  async fn shutdown(&self) -> ProviderResult<()> {
    Ok(())
  }
}
