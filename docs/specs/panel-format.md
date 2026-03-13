# Panel Format Specification

Status: Draft  
Version: 0.1  
Owner: Panel Editor  
Last Updated: 2026-03-12

---

## Purpose

Defines how panels (sensor dashboards) are stored and exchanged.

Panels describe:

-   layout
-   widgets
-   sensor bindings
-   theme selection

Panels should be portable and shareable.

## File Extension

Recommended extension:

.panel.json

Example:

mission-control.panel.json

## Top-Level Structure

Example:

{ "version": 1, "name": "Mission Control", "theme": "default-dark",
"grid": { "columns": 12, "rowHeight": 80, "spacing": 8 }, "widgets":
\[\] }

## Widget Definition

Each widget defines:

-   widget type
-   layout position
-   sensor bindings
-   widget configuration

Example:

{ "id": "cpu_load", "type": "stat", "x": 0, "y": 0, "w": 3, "h": 2,
"bindings": { "value": "cpu.total.utilization" } }

## Grid Layout

Panels use a grid system.

Fields:

columns --- number of grid columns\
rowHeight --- pixel height of a row\
spacing --- spacing between widgets

Example:

{ "columns": 12, "rowHeight": 80, "spacing": 8 }

## Widget Positioning

Widget layout fields:

x --- grid column position\
y --- grid row position\
w --- width in columns\
h --- height in rows

Example:

{ "x": 3, "y": 0, "w": 3, "h": 2 }

## Sensor Binding

Widgets reference sensors using a binding key.

Example:

"value": "cpu.total.utilization"

Bindings should be resolved by the runtime through the sensor store.

## Versioning

Panels include a version number to allow migration.

Example:

"version": 1
