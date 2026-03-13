# AstraGauge Widget Development Guidelines

## Purpose

Defines rules for building widgets that integrate cleanly with
AstraGauge.

Widgets should be:

-   reusable
-   visually consistent
-   lightweight

## Widget Responsibilities

A widget should:

-   display data
-   react to value updates
-   respect theme styling

Widgets should NOT:

-   access providers directly
-   store persistent data
-   manage global layout

## Standard Widget Anatomy

Each widget should contain:

Label\
Primary Value\
Optional Unit\
Optional Trend Visualization

Example:

CPU Load 43% ▁▂▄▆▅▃▂▁

## Data Input

Widgets receive data through bindings.

Example binding:

"value": "cpu.total.utilization"

Widgets should treat input as reactive values.

## Layout Constraints

Widgets must support:

-   dynamic resizing
-   grid-based placement

Widgets should adapt their layout depending on size.

Small: value + label

Medium: value + label + unit

Large: value + label + sparkline

## Performance

Widgets should:

-   avoid heavy computation
-   render efficiently
-   minimize redraws

Recommended update behavior:

only re-render when value changes.

## Styling

Widgets must not hardcode colors.

All styling should come from the theme system.

Examples:

theme.surface\
theme.text_primary\
theme.accent

## Accessibility

Widgets should support:

-   high contrast themes
-   clear numeric display
-   readable typography

## Goal

Widgets should behave like small instruments that fit naturally into
panels.
