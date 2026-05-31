# Feature Specification: Domain Types and Validation

**Feature Branch**: `feature/time-machine-domain-types`

**Created**: 2026-05-30

**Status**: Draft

**Input**: Feature: Domain Types and Validation. Description: Core type definitions and validation logic for sensors, providers, widgets, panels, and themes. Relevant files: crates/domain/src/lib.rs, sensor.rs, provider.rs, widget.rs, panel.rs, theme.rs, validation.rs. Focus on this feature only; do not modify other features.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Provider Developer Registers New Sensors (Priority: P1)

A provider developer creates a new system provider (e.g., for GPU monitoring). They define `SensorDescriptor` entries for each metric their provider exposes (e.g., `gpu.temperature`, `gpu.vram.used`). The `SensorId` validation ensures their IDs follow the project convention (lowercase, dot-separated, 2-4 segments). Their `ProviderManifest` declares capabilities, sensor categories, and metadata — validated before registration.

**Why this priority**: Provider development is the primary extensibility mechanism. Without valid sensor IDs and manifests, no data flows through the system. This is the foundational contract.

**Independent Test**: Can be fully tested by creating a ProviderManifest with valid and invalid sensor IDs, verifying acceptance/rejection, and confirming JSON/TOML roundtrip serialization.

**Acceptance Scenarios**:

1. **Given** a provider developer defines sensor IDs like `gpu.temperature`, **When** they create SensorId instances, **Then** validation passes and IDs are usable as keys in the sensor store.
2. **Given** a provider developer creates a ProviderManifest in TOML, **When** they parse it with `toml::from_str`, **Then** all fields deserialize correctly including nested capabilities and sensor categories.
3. **Given** a provider developer submits a manifest with an empty `id` field, **When** they call `manifest.validate()`, **Then** a `DomainError::InvalidFormat` is returned.

---

### User Story 2 - Widget Author Defines Widget Manifest (Priority: P2)

A widget author creates a new widget type (e.g., a gauge or sparkline). They write a `WidgetManifest` defining sizing constraints, configurable properties, binding targets, preview configuration, theming support, capabilities, and validation rules. The manifest is the single source of truth for how the runtime and editor treat their widget.

**Why this priority**: Widget manifests are the second core extensibility point. They define the contract between widget code and the runtime/editor, but depend on sensor types being defined first.

**Independent Test**: Can be fully tested by parsing JSON widget manifests and verifying all fields deserialize, including complex nested types like binding targets, theming rules, and validation constraints.

**Acceptance Scenarios**:

1. **Given** a widget author writes a stat-tile manifest in JSON, **When** they parse it, **Then** all sizing, property, binding, preview, theming, capability, and validation fields are correctly deserialized.
2. **Given** a widget author omits optional fields (description, max_w/max_h), **When** they parse a minimal manifest, **Then** defaults are applied (None, empty Vec) without error.
3. **Given** a widget author defines a sparkline with `series<number>` binding type, **When** they parse the manifest, **Then** the `BindingValueType::SeriesNumber` variant is correctly matched.

---

### User Story 3 - Dashboard User Creates Panel Layout (Priority: P3)

A dashboard user creates a `.panel.json` file defining their instrument panel layout. They specify a grid configuration, theme selection, and place widgets at specific grid coordinates with sensor bindings.

**Why this priority**: Panel documents are the user-facing artifact that brings providers and widgets together, but they depend on both being defined.

**Independent Test**: Can be fully tested by parsing a panel JSON file and verifying grid config, widget placements with bindings, and theme selection.

**Acceptance Scenarios**:

1. **Given** a user creates a panel with 12-column grid and places a stat widget at (0,0) with size (3,2), **When** they parse the panel JSON, **Then** the GridConfig, WidgetPlacement, and bindings are correctly deserialized.
2. **Given** a user creates a panel without specifying a theme, **When** they parse it, **Then** theme is None and widgets is an empty Vec.

---

### User Story 4 - Theme Author Creates Visual Theme (Priority: P4)

A theme author creates a theme document defining color roles, typography choices, and spacing parameters. Both light and dark themes follow the same schema with different color values.

**Why this priority**: Theme support is required by the constitution but is consumed by the rendering layer, not by data flow.

**Independent Test**: Can be fully tested by parsing theme JSON/TOML and verifying all color roles, typography settings, and spacing parameters.

**Acceptance Scenarios**:

1. **Given** a theme author defines a dark theme with specific hex color roles, **When** they parse it, **Then** all 8 color roles are correctly deserialized.
2. **Given** a theme document is serialized to JSON and back, **When** roundtripped, **Then** it is identical to the original.

### Edge Cases

- What happens when a SensorId contains unicode characters, emojis, or mixed case? → Rejected with descriptive DomainError.
- What happens when a ProviderManifest has empty version or runtime strings? → Rejected by validate().
- What happens when a WidgetManifest has empty properties/bindings arrays? → Accepted (they are optional with defaults).
- What happens when a PanelDocument has overlapping widget placements? → Accepted (overlap detection is a runtime/editor concern, not domain).
- What happens when sensor IDs have 5+ segments like `a.b.c.d.e`? → Rejected (max 4 segments).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a validated `SensorId` type that enforces lowercase, dot-separated, 2-4 segment format
- **FR-002**: System MUST provide a `SensorDescriptor` type with id, name, category, unit, and optional device/tags
- **FR-003**: System MUST provide a `SensorSample` type with sensor_id, timestamp_ms, and optional value (nullable for missing sensors)
- **FR-004**: System MUST provide a `ProviderManifest` type parseable from both JSON and TOML with metadata, capabilities, and sensor categories
- **FR-005**: System MUST validate ProviderManifest for non-empty id, version, and runtime fields
- **FR-006**: System MUST provide a `WidgetManifest` type with sizing, properties, bindings, preview, theming, capabilities, and validation rules
- **FR-007**: System MUST support widget sizing constraints including min/max dimensions and resize modes (Fixed, Responsive, AspectLocked)
- **FR-008**: System MUST support binding value types: number, string, boolean, series\<number\>, color_role, state, timestamp
- **FR-009**: System MUST provide a `PanelDocument` type with grid config, theme selection, and widget placements with bindings
- **FR-010**: System MUST provide a `ThemeDocument` type with 8 semantic color roles, typography, and spacing
- **FR-011**: System MUST provide a `DomainError` enum covering InvalidSensorId, InvalidFormat, and ParseError variants
- **FR-012**: All types MUST support serde Serialize/Deserialize for JSON interchange
- **FR-013**: All types MUST support Clone, Debug, and PartialEq for testing and runtime use

### Key Entities

- **SensorId**: Validated string wrapper enforcing `device.metric` format (2-4 lowercase dot-separated segments)
- **SensorDescriptor**: Full sensor metadata (ID, name, category, unit, optional device and tags)
- **SensorSample**: A timestamped sensor reading with optional value (None = unavailable)
- **ProviderManifest**: Provider declaration with capabilities, sensor categories, metadata (parseable from TOML/JSON)
- **WidgetManifest**: Widget declaration with sizing, properties, bindings, preview, theming, capabilities, validation
- **PanelDocument**: Dashboard layout with grid config, theme, and widget placements
- **ThemeDocument**: Visual theme with color roles, typography, and spacing
- **DomainError**: Typed error enum for validation failures

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 7 source files compile without warnings with `cargo build`
- **SC-002**: All existing unit tests pass (37 tests across all modules) with `cargo test`
- **SC-003**: All types are accessible from the crate root via `pub use` (verified by reexport_tests)
- **SC-004**: JSON roundtrip serialization works for all types
- **SC-005**: TOML roundtrip serialization works for ProviderManifest and ThemeDocument
- **SC-006**: SensorId rejects all invalid inputs (empty, uppercase, unicode, wrong segment count, special chars)

## Assumptions

- Sensor ID format follows the convention in docs/specs/sensor-schema.md (device.metric, lowercase, 2-4 segments)
- Panel file format follows docs/specs/panel-format.md (.panel.json, grid-based)
- Widget manifest format follows docs/specs/widget-manifest.md
- Theme documents follow the design system in docs/project/design-system.md
- All types are pure data structures with no side effects or I/O
- The crate has zero external dependencies beyond serde, serde_json, and toml
- Validation is limited to structural/format rules — semantic validation (e.g., binding resolution) belongs to other crates
