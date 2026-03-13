# Sensor Store Specification

Status: Draft  
Version: 0.1  
Owner: Runtime  
Last Updated: 2026-03-12

---

# 1. Purpose

The **Sensor Store** is the runtime component responsible for managing
sensor data produced by providers and consumed by widgets through the
binding engine.

It acts as the central data hub of AstraGauge.

Responsibilities include:

-   storing the latest sensor values
-   maintaining sensor descriptors
-   managing update propagation
-   optionally buffering short history for widgets
-   providing query and subscription interfaces
-   supporting efficient runtime updates

Without a structured sensor store, providers, widgets, and bindings
would become tightly coupled, which leads to fragile architecture.

------------------------------------------------------------------------

# 2. Architectural Role

The Sensor Store sits between providers and the binding engine.

    Providers → Sensor Store → Binding Engine → Widgets

Responsibilities by layer:

  Layer            Responsibility
  ---------------- ---------------------------
  Provider         Collect sensor samples
  Sensor Store     Store and publish samples
  Binding Engine   Transform values
  Widget Runtime   Render results

------------------------------------------------------------------------

# 3. Core Concepts

## 3.1 Sensor Descriptor

A **Sensor Descriptor** describes a sensor's identity and metadata.

Example fields:

  Field      Description
  ---------- -----------------------------
  id         canonical sensor identifier
  name       human readable name
  category   sensor category
  unit       measurement unit
  device     optional device identifier
  tags       optional metadata tags

Example:

``` json
{
  "id": "cpu.package.temperature",
  "name": "CPU Package Temperature",
  "category": "temperature",
  "unit": "celsius",
  "device": "cpu0",
  "tags": ["thermal"]
}
```

Descriptors should be stable for the lifetime of a provider session.

------------------------------------------------------------------------

## 3.2 Sensor Sample

A **Sensor Sample** represents a single measurement.

Example structure:

``` json
{
  "sensor_id": "cpu.package.temperature",
  "timestamp_ms": 1712345678,
  "value": 72.3
}
```

Fields:

  Field          Description
  -------------- ---------------------
  sensor_id      canonical sensor ID
  timestamp_ms   sample timestamp
  value          measured value

------------------------------------------------------------------------

# 4. Store Data Model

The Sensor Store should maintain two primary structures.

## 4.1 Descriptor Registry

Holds metadata for all known sensors.

    Map<SensorId, SensorDescriptor>

This registry is populated when providers register sensors.

------------------------------------------------------------------------

## 4.2 Current Value Table

Stores the latest sample for each sensor.

    Map<SensorId, SensorSample>

This allows fast lookup for bindings and widgets.

------------------------------------------------------------------------

# 5. Optional History Buffer

Some widgets require short-term historical values.

Examples:

-   sparklines
-   trend indicators
-   rate calculations

The store may maintain a ring buffer per sensor.

Example:

    Map<SensorId, RingBuffer<SensorSample>>

Recommended default:

-   60--300 samples per sensor

This provides short history without turning AstraGauge into a metrics
database.

------------------------------------------------------------------------

# 6. Update Flow

Typical update pipeline:

1.  provider polls hardware
2.  provider produces sensor samples
3.  store updates latest value
4.  history buffer updated (optional)
5.  store notifies subscribers
6.  binding engine recomputes affected values
7.  widgets update

Diagram:

    Provider → Sample → Sensor Store → Subscribers → Binding Engine → Widgets

------------------------------------------------------------------------

# 7. Subscription Model

Widgets and bindings should not poll sensors directly.

Instead, they subscribe to updates.

Example conceptual API:

    subscribe(sensor_id, callback)
    unsubscribe(subscription_id)

Subscriptions may also support patterns.

Example:

    subscribe("cpu.core*.temperature")

This allows aggregation bindings to react to multiple sensors.

------------------------------------------------------------------------

# 8. Update Batching

Providers may emit multiple samples at once.

Example:

``` json
[
  {"sensor_id":"cpu.utilization","value":43.2},
  {"sensor_id":"memory.used","value":14.5}
]
```

The store should batch updates to avoid excessive UI refreshes.

Recommended strategy:

-   group updates within a small window (e.g., 16--50ms)
-   notify subscribers once per batch

This keeps the UI responsive even with high-frequency sensors.

------------------------------------------------------------------------

# 9. Update Frequency Considerations

Sensors update at different rates.

Examples:

  Sensor               Typical Rate
  -------------------- --------------
  CPU utilization      1--10 Hz
  GPU metrics          1--5 Hz
  temperatures         0.5--2 Hz
  network throughput   1--5 Hz

The store must support mixed update frequencies efficiently.

------------------------------------------------------------------------

# 10. Provider Integration

Providers interact with the store through a registration API.

Example lifecycle:

    register_provider()
    register_sensor()
    push_sample()
    shutdown_provider()

Providers should register sensors before pushing samples.

------------------------------------------------------------------------

# 11. Query Interface

The store should support basic queries used by the editor and runtime.

Examples:

Get descriptor:

    get_sensor_descriptor(sensor_id)

Get current value:

    get_sensor_value(sensor_id)

List sensors:

    list_sensors()

Filter sensors:

    list_sensors(category="temperature")

------------------------------------------------------------------------

# 12. Pattern Queries

The store should support wildcard pattern matching.

Example:

    cpu.core*.temperature

Resolved sensors:

    cpu.core0.temperature
    cpu.core1.temperature
    cpu.core2.temperature

This capability supports aggregation bindings.

------------------------------------------------------------------------

# 13. Error Handling

The store must gracefully handle:

  Situation          Behavior
  ------------------ -------------------------
  missing sensor     return null / undefined
  stale sensor       mark timestamp
  provider failure   keep last known value
  invalid sample     discard and log

Widgets should display fallback states such as:

    N/A

rather than crashing the runtime.

------------------------------------------------------------------------

# 14. Staleness Detection

Sensors may stop updating.

The store should track staleness.

Example rule:

    if current_time - last_sample > threshold
        mark sensor stale

Widgets may visually indicate stale values.

------------------------------------------------------------------------

# 15. Threading Model

The store should support concurrent updates safely.

Possible strategy:

-   provider updates on worker threads
-   store writes synchronized
-   UI reads on main thread
-   update notifications dispatched asynchronously

The exact concurrency model may depend on runtime language and platform.

------------------------------------------------------------------------

# 16. Memory Considerations

Sensor counts may vary widely.

Typical desktop:

  Metric               Estimate
  -------------------- ----------
  sensor descriptors   50--500
  active sensors       50--200
  history buffers      optional

The store should be lightweight and predictable.

Avoid unbounded growth.

------------------------------------------------------------------------

# 17. Editor Integration

The editor relies on the sensor store for:

-   listing sensors for bindings
-   previewing live values
-   showing sensor categories
-   testing bindings

The editor may also support **mock sensor data** using the same store
interface.

------------------------------------------------------------------------

# 18. Mock Data Mode

The runtime may run in mock mode.

Example behavior:

-   generate synthetic sensor values
-   simulate trends
-   populate descriptors

This allows the editor to work even without providers.

------------------------------------------------------------------------

# 19. Performance Goals

The sensor store should support:

-   hundreds of sensors
-   frequent updates
-   fast lookup
-   low memory overhead

Performance targets:

-   constant-time value lookup
-   efficient pattern resolution
-   minimal UI thread blocking

------------------------------------------------------------------------

# 20. Future Extensions

Potential future capabilities:

-   longer historical storage
-   rate calculations
-   sensor groups
-   persistence across sessions
-   distributed providers
-   remote sensors

These should be added carefully to avoid turning AstraGauge into a full
telemetry database.

------------------------------------------------------------------------

# 21. Summary

The AstraGauge Sensor Store provides a centralized runtime model for
sensor data.

It ensures:

-   clean separation between providers and widgets
-   efficient update propagation
-   consistent sensor access
-   support for binding and aggregation
-   optional short-term history

A well-defined store prevents tight coupling and enables AstraGauge to
scale cleanly as providers and widgets grow.
