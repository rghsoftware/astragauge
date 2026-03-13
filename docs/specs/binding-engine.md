# Binding Engine Specification

Status: Draft  
Version: 0.1  
Owner: Runtime  
Last Updated: 2026-03-12

---

## Purpose

The Binding Engine connects **sensors** to **widgets**.

It is responsible for:

-   resolving sensor identifiers
-   transforming sensor values
-   applying aggregations
-   formatting values for display
-   routing updates to widgets

The binding engine acts as the **data translation layer** between the
Sensor Store and the UI runtime.

------------------------------------------------------------------------

# Architecture Role

    Providers → Sensor Store → Binding Engine → Widgets

Responsibilities:

  Layer            Responsibility
  ---------------- ------------------------------
  Provider         acquire sensor values
  Sensor Store     hold latest values
  Binding Engine   resolve and transform values
  Widget           render values

------------------------------------------------------------------------

# Binding Model

Bindings connect widget properties to sensors.

Example:

    widget.value ← cpu.utilization

Bindings are declared in the panel configuration.

------------------------------------------------------------------------

# Basic Binding

Example widget configuration:

``` json
{
  "type": "gauge",
  "bindings": {
    "value": "cpu.utilization"
  }
}
```

The binding engine resolves the sensor ID and provides the value to the
widget.

------------------------------------------------------------------------

# Binding Targets

Widgets expose bindable properties.

Common targets:

  Property   Purpose
  ---------- -----------------------
  value      primary numeric value
  min        minimum scale
  max        maximum scale
  label      widget label
  color      visual state

Example:

``` json
{
  "bindings": {
    "value": "cpu.utilization",
    "max": "cpu.max_utilization"
  }
}
```

------------------------------------------------------------------------

# Transformations

Bindings may apply transforms to raw sensor values.

Example:

    value = percent(cpu.utilization)

Supported transform categories:

  Transform   Purpose
  ----------- ----------------------
  scale       convert units
  percent     normalize value
  clamp       restrict value range
  round       rounding
  abs         absolute value

Example:

    round(cpu.temperature, 1)

------------------------------------------------------------------------

# Aggregations

Some widgets require aggregated values.

Example:

    avg(cpu.core*.temperature)

Supported aggregation functions:

  Function   Description
  ---------- -------------------
  avg        average
  min        minimum
  max        maximum
  sum        sum
  count      number of sensors

Example:

    max(cpu.core*.temperature)

------------------------------------------------------------------------

# Pattern Matching

Wildcard patterns allow bindings to reference multiple sensors.

Example:

    cpu.core*.temperature

Resolved sensors:

    cpu.core0.temperature
    cpu.core1.temperature
    cpu.core2.temperature

The aggregation function determines how the values are combined.

------------------------------------------------------------------------

# Derived Bindings

Bindings may compute derived values.

Example:

    memory.utilization =
    memory.used / memory.total

Derived bindings allow panels to compute values not directly provided by
sensors.

------------------------------------------------------------------------

# Unit Conversion

Bindings may convert between units.

Example:

    fahrenheit(cpu.temperature)

The runtime should support:

-   celsius → fahrenheit
-   bytes → megabytes
-   bytes → gigabytes
-   bits → megabits

Units should remain defined in sensor descriptors.

------------------------------------------------------------------------

# Binding Lifecycle

Typical update flow:

1.  provider updates sensor
2.  sensor store updates value
3.  binding engine resolves bindings
4.  transforms applied
5.  widget updated

Diagram:

    Sensor Update
          ↓
    Sensor Store
          ↓
    Binding Engine
          ↓
    Transforms / Aggregations
          ↓
    Widget Update

------------------------------------------------------------------------

# Performance Considerations

Bindings should be evaluated efficiently.

Rules:

-   avoid recalculating unchanged bindings
-   cache resolved sensor patterns
-   batch widget updates

High-frequency sensors (e.g., 60Hz) must not stall the UI.

------------------------------------------------------------------------

# Error Handling

Binding failures should not crash the runtime.

Examples:

Invalid sensor:

    cpu.invalid.metric

Behavior:

-   log warning
-   widget shows "N/A"

------------------------------------------------------------------------

# Future Extensions

The binding engine may later support:

-   expressions
-   conditional formatting
-   threshold triggers
-   historical queries

Example future binding:

    if(cpu.temperature > 80, "critical", "normal")

------------------------------------------------------------------------

# Goals

The AstraGauge binding engine should provide:

-   flexible sensor-to-widget mapping
-   predictable performance
-   extensibility for advanced panel logic
