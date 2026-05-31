# Feature: provider-host

**Status:** In Progress  
**Branch:** feature/time-machine-provider-host  
**Depends On:** sensor-store, domain-types

---

## User Stories

### US1: Runtime Engineer
As a runtime engineer, I want provider panics to be contained so that one failing provider doesn't crash the entire application.

### US2: Application Developer
As an application developer, I want to validate provider manifests at registration time so that invalid providers are rejected early.

### US3: Dashboard Builder
As a dashboard builder, I want to query individual provider health so that I can show status per-provider in the UI.

### US4: System Integrator
As a system integrator, I want to unregister providers at runtime so that I can dynamically reconfigure the system without restart.

---

## Functional Requirements

### FR1: Manifest Validation
`register_provider` MUST call `ProviderManifest::validate()` and reject providers with invalid manifests.

### FR2: Unregister Provider
The host MUST expose `unregister_provider(id)` that stops the provider's poll task and removes it.

### FR3: Provider Health Query
The host MUST expose `get_provider_health(id)` returning the health of a single provider, or None if not found.

### FR4: Is Running Check
The host MUST expose `is_provider_running(id)` returning whether the provider's poll task is active.

### FR5: Un-ignored Panic Tests
The 3 ignored panic containment unit tests should be fixed and enabled.

### FR6: Backward Compatibility
All existing APIs and tests must continue to work.

---

## Success Criteria

1. All 8 existing tests continue to pass.
2. `register_provider` rejects providers with empty/invalid manifests.
3. `unregister_provider` stops a running provider.
4. `get_provider_health(id)` returns correct health status.
5. `is_provider_running(id)` returns correct running state.
6. Panic containment tests pass without `#[ignore]`.
7. Clippy clean, no warnings.
