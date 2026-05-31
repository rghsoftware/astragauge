# Tasks: System Providers

Feature: system-providers
Phase: tasks

---

## Tasks

- [ ] **Task 1**: MockProvider — Add configurable health state and poll failure simulation
- [ ] **Task 2**: LinuxProvider — Replace std::fs with tokio::fs for async I/O
- [ ] **Task 3**: LinuxProvider — Add configurable poll interval via constructor
- [ ] **Task 4**: LinuxProvider — Implement health check verifying /proc readability
- [ ] **Task 5**: LinuxProvider — Add swap sensors from /proc/meminfo (swap.used, swap.total, swap.utilization)
- [ ] **Task 6**: LinuxProvider — Add disk sensors from /proc/diskstats
- [ ] **Task 7**: LinuxProvider — Add network sensors from /proc/net/dev
- [ ] **Task 8**: Fix integration tests — use SensorId::new() instead of custom validation
- [ ] **Task 9**: Add new tests for all new sensor types
- [ ] **Task 10**: Verification — cargo test + clippy + build
