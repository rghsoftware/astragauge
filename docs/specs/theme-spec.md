# Theme Specification

Status: Draft  
Version: 0.1  
Owner: Theming  
Last Updated: 2026-03-12

---

## Purpose

Themes define the visual style of panels and widgets.

Themes should allow customization while preserving readability.

## Theme Structure

Example directory:

theme/ theme.toml colors.toml typography.toml widget-styles.toml

## Core Theme Fields

Example:

name = "default-dark" version = 1

## Color Roles

Themes define semantic color roles.

Example:

background = "#0F1115" surface = "#1A1E24" text_primary = "#FFFFFF"
text_secondary = "#9AA4B2" accent = "#4FA3FF" good = "#4CD964" warn =
"#FFC857" critical = "#FF4D4D"

Widgets should reference roles instead of raw colors.

## Typography

Themes define font families and sizes.

Example:

primary_font = "Inter" numeric_font = "JetBrains Mono"

## Spacing

Themes define spacing rules.

Example:

grid_spacing = 8 widget_padding = 12

## Widget Styles

Themes may override widget appearance.

Example:

gauge_thickness = 6 sparkline_stroke = 2

## Goals

Themes should allow visual creativity while preserving:

-   legibility
-   consistency
-   panel structure
