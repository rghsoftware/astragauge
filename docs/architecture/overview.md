# AstraGauge Architecture Overview

## Document Status

-   Project: AstraGauge
-   Document Type: Architecture Overview
-   Version: v0.1
-   Status: Draft

------------------------------------------------------------------------

# 1. Purpose

This document provides a **high‑level map of the AstraGauge
architecture** and links together the individual specifications that
define the platform.

It answers the question:

**How do all the pieces of AstraGauge fit together?**

This overview is intentionally conceptual. Detailed behavior is defined
in the individual specifications referenced throughout this document.

------------------------------------------------------------------------

# 2. System Concept

AstraGauge is a **modular system instrumentation platform** designed to
render customizable dashboards for local system metrics.

The architecture separates concerns into distinct layers:

-   **Providers** collect metrics
-   **Sensor Store** manages runtime data
-   **Binding Engine** transforms data
-   **Widgets** render visual components
-   **Panels** compose widgets
-   **Themes** control appearance
-   **Editor** allows panel creation
-   **Runtime** executes everything

This layered model prevents tight coupling between components.

------------------------------------------------------------------------

# 3. Core Data Flow

The primary runtime pipeline:

Providers → Sensor Store → Binding Engine → Widgets → Panels → Renderer

Explanation:

1.  Providers gather sensor data
2.  Sensor Store records samples
3.  Binding Engine resolves widget inputs
4.  Widgets render values
5.  Panels arrange widgets
6.  Renderer displays the result

------------------------------------------------------------------------

# 4. Major Subsystems

## 4.1 Provider System

Providers collect metrics from operating systems and hardware sources.

Examples:

-   CPU metrics
-   GPU metrics
-   memory usage
-   disk throughput
-   network statistics

Relevant specs:

-   provider API
-   provider packaging

------------------------------------------------------------------------

## 4.2 Sensor Model

Sensors represent measurable system values.

Each sensor has:

-   descriptor metadata
-   a unique ID
-   a measurement unit
-   a current sample value

Relevant specs:

-   sensor schema
-   sensor store

------------------------------------------------------------------------

## 4.3 Data Transformation

The Binding Engine connects sensors to widget inputs.

It handles:

-   sensor lookup
-   wildcard resolution
-   transformations
-   aggregations
-   derived metrics

Example binding:

cpu.utilization → gauge_widget.value

Relevant spec:

-   binding engine

------------------------------------------------------------------------

## 4.4 Widget System

Widgets render visual representations of data.

Examples:

-   gauges
-   graphs
-   numeric displays
-   bars
-   sparklines

Widgets are defined through a **manifest specification** that allows the
editor and runtime to understand their capabilities.

Relevant specs:

-   widget manifest
-   widget guidelines

------------------------------------------------------------------------

## 4.5 Panel System

Panels define layouts of widgets.

Panels describe:

-   widget placement
-   widget properties
-   bindings
-   layout configuration

Panels are the primary user artifact.

Relevant spec:

-   panel format

------------------------------------------------------------------------

## 4.6 Theme System

Themes control visual appearance.

Themes define:

-   color palettes
-   typography
-   surface styles
-   semantic UI tokens

Widgets reference theme roles rather than hard-coded colors.

Relevant spec:

-   theme specification

------------------------------------------------------------------------

## 4.7 Runtime System

The runtime orchestrates the entire execution environment.

Major runtime services include:

-   Provider Host
-   Sensor Store
-   Binding Engine
-   Widget Registry
-   Theme Engine
-   Panel Session Manager
-   Renderer

Relevant spec:

-   runtime architecture

------------------------------------------------------------------------

## 4.8 Panel Editor

The editor allows users to visually design dashboards.

Editor features include:

-   canvas layout editing
-   widget property inspection
-   binding configuration
-   theme selection
-   live preview

The editor embeds the runtime for preview.

Relevant spec:

-   panel editor architecture

------------------------------------------------------------------------

# 5. Architectural Layers

The system can be understood as layered architecture.

Infrastructure Layer - providers - sensor store

Data Layer - sensor descriptors - sensor samples

Logic Layer - binding engine

Presentation Layer - widgets - panels - themes

Application Layer - runtime - editor

------------------------------------------------------------------------

# 6. Specification Map

The following documents define the AstraGauge platform.

Architecture

-   runtime architecture
-   panel editor architecture

Core Specifications

-   sensor schema
-   sensor store
-   binding engine

UI Specifications

-   widget manifest
-   panel format
-   theme spec

Ecosystem Specifications

-   provider API
-   provider packaging

Development Guides

-   widget guidelines
-   provider guidelines

------------------------------------------------------------------------

# 7. Runtime Execution Model

When AstraGauge starts:

1.  runtime loads configuration
2.  providers are discovered
3.  sensors are registered
4.  sensor store begins receiving updates
5.  panel document is loaded
6.  widgets are instantiated
7.  bindings are evaluated
8.  renderer displays the panel

Updates then propagate through the pipeline as sensor values change.

------------------------------------------------------------------------

# 8. Design Principles

The AstraGauge architecture follows several core principles.

Separation of concerns\
Providers, widgets, and runtime components remain independent.

Schema‑driven systems\
Editors and runtime behavior rely on manifests and schemas rather than
hard-coded logic.

Unidirectional data flow\
Data moves predictably through the pipeline.

Extensibility\
Providers and widgets can be added without modifying the core runtime.

Theming by tokens\
Themes define semantic styling roles rather than raw colors.

------------------------------------------------------------------------

# 9. Future Architectural Areas

Areas expected to evolve:

-   multi-panel runtime
-   remote sensor providers
-   provider sandboxing
-   plugin marketplaces
-   historical data support
-   automation rules

These extensions should build upon the current architecture rather than
replacing it.

------------------------------------------------------------------------

# 10. Summary

AstraGauge is designed as a **modular instrumentation platform** with
clearly separated layers:

Providers → Sensor Store → Binding Engine → Widgets → Panels → Renderer

This architecture enables:

-   flexible system monitoring
-   portable dashboards
-   strong editor support
-   a growing ecosystem of providers and widgets

The individual specifications referenced throughout this document define
the detailed contracts for each subsystem.
