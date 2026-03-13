# AstraGauge Branding Guide

## Project Name

**AstraGauge**

## Tagline

**Instrument your system. Style it your way.**

## Short Description

AstraGauge is a cross‑platform runtime for building customizable system
sensor panels for monitoring hardware and system metrics.

## Positioning

AstraGauge sits between two existing categories:

-   **Enterprise telemetry dashboards** (Grafana, Prometheus)
-   **Consumer system monitors** (AIDA64 sensor panels, HWInfo, btop)

AstraGauge focuses on **local system instrumentation with strong visual
customization**.

## Core Concepts

  -----------------------------------------------------------------------
  Concept                             Definition
  ----------------------------------- -----------------------------------
  Provider                            A module that discovers and reads
                                      sensors

  Sensor                              A measurable value such as
                                      temperature, utilization, or power

  Widget                              A UI component that visualizes a
                                      sensor

  Panel                               A layout of widgets forming a
                                      dashboard

  Theme                               A style definition controlling
                                      appearance
  -----------------------------------------------------------------------

## Messaging

Preferred terminology:

-   system sensors
-   sensor panels
-   system instrumentation
-   hardware metrics
-   customizable panels

Avoid emphasizing:

-   telemetry platform
-   observability stack
-   monitoring pipeline

These imply infrastructure tooling rather than a local instrumentation
tool.

## Elevator Pitch

AstraGauge is a modular system sensor panel runtime that lets users
build highly customizable displays for hardware and system metrics.

## Product Identity

Tone should feel:

-   precise
-   technical
-   modern
-   calm

Avoid:

-   gamer RGB branding
-   industrial SCADA aesthetic
-   enterprise DevOps language

## Visual Identity

Visual direction should evoke:

-   instrumentation
-   constellations
-   precision measurement

Possible motifs:

-   circular gauges
-   constellation‑like dots
-   clean geometric lines

Avoid:

-   rockets
-   flame icons
-   esports styling

## Naming Conventions

Recommended crate naming pattern:

astragauge-`<domain>`{=html}

Examples:

-   astragauge-core
-   astragauge-runtime
-   astragauge-provider-api
-   astragauge-widgets
-   astragauge-provider-linux

## Repo Naming

Recommended repository name:

astragauge

## Binary Naming

Primary executable:

astragauge

Additional binaries only if needed:

-   astragauge-editor
-   astragauge-agent
