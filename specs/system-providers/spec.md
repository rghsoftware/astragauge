# Feature: System Providers

Feature ID: system-providers
Status: in-progress
Branch: feature/time-machine-system-providers
Depends on: domain-types, provider-host

---

## Summary

Enhance the providers crate with production-quality system providers. The crate currently has a functional MockProvider and LinuxProvider, but both need improvements for reliability, testability, and completeness.

---

## User Stories

### 1. Runtime Developer
As a runtime developer, I want providers that use async I/O so the application doesn't block on filesystem reads during polling.

### 2. Panel Author
As a panel author, I want comprehensive sensor coverage (CPU, memory, swap, disk, network, temperature) so I can build rich monitoring dashboards.

### 3. Test Engineer
As a test engineer, I want MockProvider to simulate failure modes (degraded health, poll failures) so I can verify runtime resilience.

### 4. System Integrator
As a system integrator, I want a configurable poll interval per provider so I can balance update frequency against resource usage.

---

## Current State

### MockProvider
- Returns configurable sensor descriptors and values
- Fixed manifest (id="mock.provider")
- Health always returns Ok
- 7 unit tests

### LinuxProvider (#[cfg(target_os = "linux")])
- CPU utilization from /proc/stat (delta-based calculation)
- Memory sensors from /proc/meminfo (used, total, available, utilization)
- Temperature sensors from /sys/class/hwmon
- Uses blocking std::fs (not async-safe)
- Hardcoded 1000ms poll interval
- Health always returns Ok
- 5 unit tests + 6 ignored integration tests

---

## Requirements

### Functional

1. **FR-1**: MockProvider must support configurable health state (Ok, Degraded, Error)
2. **FR-2**: MockProvider must support configurable poll failure simulation
3. **FR-3**: LinuxProvider must use async filesystem I/O (tokio::fs)
4. **FR-4**: LinuxProvider poll interval must be configurable via constructor
5. **FR-5**: LinuxProvider health() must verify /proc/stat and /proc/meminfo are readable
6. **FR-6**: LinuxProvider must expose swap sensors (swap.used, swap.total, swap.utilization)
7. **FR-7**: LinuxProvider must expose disk sensors from /proc/diskstats (disk.X.read_bytes, disk.X.write_bytes)
8. **FR-8**: LinuxProvider must expose network sensors from /proc/net/dev (network.X.rx_bytes, network.X.tx_bytes)
9. **FR-9**: All sensor IDs must conform to sensor-schema.md conventions
10. **FR-10**: Integration tests must use SensorId::new() for validation (not custom checks)

### Non-Functional

11. **NFR-1**: File I/O errors must never panic — each sensor category fails independently
12. **NFR-2**: Blocking I/O during polling must not stall other providers
13. **NFR-3**: New sensor types must follow the bounded-memory principle

---

## Success Criteria

- All existing tests pass
- MockProvider supports health and failure configuration
- LinuxProvider uses tokio::fs for all reads
- At least CPU, memory, swap, disk, network, and temperature sensor categories work
- Health check verifies procfs availability
- clippy clean, no warnings
