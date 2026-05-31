# Feature: sensor-store

**Status:** In Progress  
**Branch:** feature/time-machine-sensor-store  
**Depends On:** domain-types

---

## User Stories

### US1: Provider Developer
As a provider developer, I want to register sensors and push samples so that my hardware data flows into the system.

### US2: Widget Author
As a widget author, I want to subscribe to sensor updates (including wildcard patterns) so that my widget reacts to live data without polling.

### US3: Runtime Engineer
As a runtime engineer, I want the store to batch notifications so that high-frequency sensors don't overwhelm the UI.

### US4: Dashboard Builder
As a dashboard builder, I want to query sensors by category and wildcard pattern so that I can discover and bind available metrics.

---

## Functional Requirements

### FR1: Subscription Integration
The SensorStore MUST integrate SubscriptionManager so that pushing samples automatically notifies subscribers matching the sample's sensor_id.

### FR2: Subscribe/Unsubscribe on Store
The store MUST expose `subscribe(pattern) -> Subscription` and `unsubscribe(id)` methods that delegate to the internal SubscriptionManager.

### FR3: Batch Notification
When `push_samples` is called with multiple samples, the store MUST notify subscribers once per affected pattern (not once per sample).

### FR4: Single Push Notification
When `push_sample` is called, the store MUST notify matching subscribers immediately.

### FR5: List Sensors by Category
The store MUST expose `list_sensors_by_category(category: &str)` returning sensors matching that category.

### FR6: Pattern Query
The store MUST expose `query_pattern(pattern: &str)` returning SensorIds matching the wildcard pattern against registered sensors.

### FR7: Staleness Batch Check
The store MUST expose `get_stale_sensors(now_ms: u64)` returning all SensorIds that are currently stale.

### FR8: Sensor Count
The store MUST expose `sensor_count()` returning the number of registered sensors.

### FR9: Thread Safety
All public methods must remain safe for concurrent use (already ensured by RwLock, must be preserved).

### FR10: Backward Compatibility
Existing API (`register_sensor`, `unregister_sensor`, `push_sample`, `push_samples`, `get_value`, `get_value_with_timestamp`, `is_stale`, `get_history`, `get_descriptor`, `list_sensors`) MUST continue to work identically.

---

## Success Criteria

1. All 57 existing tests continue to pass.
2. New subscription integration tests pass (subscribe → push → receive).
3. `push_samples` with 10+ samples triggers batched notification.
4. `list_sensors_by_category` filters correctly.
5. `query_pattern` returns correct wildcard matches from store.
6. `get_stale_sensors` returns only stale sensors.
7. Clippy clean, no warnings.
