# AstraGauge Provider Development Guidelines

## Purpose

Providers discover and read sensors from a system or device.

Examples:

-   Linux system sensors
-   Windows performance counters
-   GPU metrics
-   hardware monitoring libraries

## Responsibilities

Providers must:

-   discover sensors
-   read sensor values
-   normalize output

Providers should NOT:

-   manage UI
-   store sensor history
-   perform rendering

## Provider Lifecycle

Typical lifecycle:

initialize\
discover sensors\
poll sensors\
shutdown

## Sensor Discovery

Providers return SensorDescriptor objects.

Each descriptor describes:

-   sensor id
-   name
-   category
-   unit
-   device (optional)
-   tags

Example:

{ "id": "cpu.temp", "name": "CPU Temperature", "category":
"temperature", "unit": "celsius" }

## Polling

Providers periodically return sensor samples.

Example:

{ "sensor_id": "cpu.temp", "timestamp_ms": 1712341234, "value": 72.5 }

Polling frequency should be configurable.

## Error Handling

Providers should:

-   report health state
-   gracefully handle sensor failure
-   avoid crashing the runtime

## Naming Guidelines

Sensor IDs should follow:

device.metric

Examples:

cpu.total.utilization\
gpu.temperature\
memory.used

## Goal

Providers should provide reliable normalized sensor data for the
runtime.
