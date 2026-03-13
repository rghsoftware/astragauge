# AstraGauge Design System

## Design Philosophy

AstraGauge panels should feel like **instrument panels**, not
dashboards.

The interface should prioritize:

-   clarity
-   legibility
-   composability
-   restrained styling

The goal is to avoid both:

-   chaotic gamer‑style sensor panels
-   overly stiff industrial monitoring UIs

## Core Principles

### Instrument First

Widgets should behave like measurement instruments.

Each widget should clearly present:

-   the measured value
-   context for the value
-   optional historical trend

### Calm Density

Panels should support dense layouts while remaining readable.

Avoid:

-   excessive borders
-   heavy gradients
-   unnecessary decoration

### Distance Legibility

Many panels are viewed from a distance (secondary monitors, small LCD
screens).

Minimum recommended sizes:

  Element           Size
  ----------------- -----------
  Primary value     36--72 px
  Secondary value   16--24 px
  Labels            12--16 px

## Layout System

Use a grid‑based layout.

Recommended base spacing:

8px grid

Major spacing:

24px

Rules:

-   widgets snap to grid
-   consistent padding between widgets
-   align numeric values vertically when possible

## Typography

Recommended fonts:

Labels: - Inter - IBM Plex Sans

Numeric values: - JetBrains Mono - IBM Plex Mono

Using monospaced numbers prevents jitter when values change.

## Color System

Themes should define semantic color roles rather than raw colors.

Core roles:

  Role             Purpose
  ---------------- -------------------
  background       panel background
  surface          widget background
  text-primary     main value text
  text-secondary   labels
  accent           highlights
  good             normal state
  warn             caution
  critical         alert

Example palette:

background: #0F1115\
surface: #1A1E24\
accent: #4FA3FF\
good: #4CD964\
warn: #FFC857\
critical: #FF4D4D

Widgets should reference color roles instead of hardcoded values.

## Widget Structure

Standard widget layout:

Label\
Primary Value\
Optional Unit\
Optional Graph

Example:

CPU Load\
43%\
▁▂▄▆▅▃▂▁

## Widget Types

Typical widget categories:

-   Stat tile
-   Gauge
-   Bar gauge
-   Sparkline
-   Sensor list

Widgets should remain visually consistent across themes.

## Theme Structure

A theme should define:

-   colors
-   typography
-   spacing
-   widget styling
-   corner radius

Example structure:

theme/ theme.toml colors.toml typography.toml widget-styles.toml

## Animation

Animations should be minimal and functional.

Acceptable:

-   value transitions
-   sparkline updates
-   gauge movement

Avoid:

-   bouncing widgets
-   flashy gradients
-   large animated elements

## Panel Composition Guidelines

Recommended panel patterns:

Minimal Panel - 4--6 widgets

Hardware Monitor - CPU - GPU - temperatures - memory

Small Display Panel - optimized for compact screens

Avoid panels with dozens of tiny widgets.

## Iconography

Icons should be:

-   simple
-   monochrome
-   line‑based

Common icons:

CPU\
GPU\
Memory\
Disk\
Network\
Temperature\
Fan

Avoid emoji or decorative icons.

## Goal

The AstraGauge design system should enable highly customizable panels
while ensuring panels remain clean, readable, and visually coherent.
