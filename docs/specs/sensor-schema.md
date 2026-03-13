# Sensor Schema Specification

Status: Draft  
Version: 0.1  
Owner: Providers  
Last Updated: 2026-03-12

---

## Purpose

Defines a consistent schema and naming convention for sensors produced
by providers.

Without a strict convention, different providers will expose
inconsistent sensor IDs, making panels difficult to share.

This specification ensures:

-   predictable sensor identifiers
-   cross-provider compatibility
-   shareable panel configurations
-   clean widget bindings

------------------------------------------------------------------------

# Core Concept

A **sensor** represents a measurable value exposed by a provider.

Examples:

-   CPU utilization
-   GPU temperature
-   memory usage
-   fan speed
-   disk throughput

Sensors must follow a consistent identifier format.

------------------------------------------------------------------------

# Sensor ID Format

The canonical format is:

device.metric

Example:

cpu.utilization gpu.temperature memory.used

------------------------------------------------------------------------

# Extended Format

When more precision is required, use:

device.component.metric

Examples:

cpu.core0.temperature cpu.core1.temperature gpu.vram.used
disk.sda.read_throughput

------------------------------------------------------------------------

# Multi-Level Format

For complex sensors:

device.component.subcomponent.metric

Examples:

gpu.vram.controller.temperature cpu.package.power
network.eth0.rx_throughput

------------------------------------------------------------------------

# Naming Rules

## Lowercase Only

All sensor IDs must use lowercase characters.

Correct:

cpu.temperature

Incorrect:

Cpu.Temperature

------------------------------------------------------------------------

## Dot Separation

Use **dot notation** for hierarchy.

Correct:

cpu.package.power

Incorrect:

cpu_package_power

------------------------------------------------------------------------

## No Spaces

Sensor IDs must never contain spaces.

Correct:

gpu.temperature

Incorrect:

gpu temperature

------------------------------------------------------------------------

## Singular Device Names

Use singular device identifiers.

Correct:

cpu.temperature

Incorrect:

cpus.temperature

------------------------------------------------------------------------

# Standard Device Names

Providers should use these canonical device names when possible.

  Device    Meaning
  --------- ---------------------
  cpu       processor metrics
  gpu       graphics processor
  memory    system RAM
  disk      storage devices
  network   network interfaces
  fan       cooling fans
  battery   battery sensors
  system    system-wide metrics

------------------------------------------------------------------------

# Standard Metric Names

These metric names should be reused across providers.

  Metric        Meaning
  ------------- --------------------
  temperature   degrees of heat
  utilization   percent usage
  frequency     clock speed
  power         watts consumed
  voltage       voltage level
  current       electrical current
  rpm           fan speed
  used          used capacity
  free          available capacity
  throughput    data transfer rate

------------------------------------------------------------------------

# Units

Units must be defined in the sensor descriptor rather than embedded in
the name.

Correct:

gpu.temperature unit: celsius

Incorrect:

gpu.temperature_celsius

------------------------------------------------------------------------

# Tags

Sensors may include optional tags for filtering.

Example:

tags = \["core", "thermal"\]

Tags should not change the canonical sensor ID.

------------------------------------------------------------------------

# Examples

## CPU Sensors

cpu.utilization cpu.package.temperature cpu.package.power
cpu.core0.temperature cpu.core1.temperature

------------------------------------------------------------------------

## GPU Sensors

gpu.utilization gpu.temperature gpu.power gpu.vram.used gpu.vram.total

------------------------------------------------------------------------

## Memory Sensors

memory.used memory.free memory.utilization

------------------------------------------------------------------------

## Disk Sensors

disk.sda.utilization disk.sda.read_throughput disk.sda.write_throughput

------------------------------------------------------------------------

## Network Sensors

network.eth0.rx_throughput network.eth0.tx_throughput

------------------------------------------------------------------------

# Provider Mapping

Providers may internally use different sensor names.

Example:

HWInfo:

CPU Package Temperature

Provider must normalize to:

cpu.package.temperature

------------------------------------------------------------------------

# Stability Requirement

Once a sensor ID is introduced, it must remain stable across versions.

Changing sensor IDs breaks existing panels.

------------------------------------------------------------------------

# Goal

The AstraGauge sensor schema ensures that:

-   widgets can bind to sensors reliably
-   panels can be shared between systems
-   providers can evolve without breaking UI configurations
