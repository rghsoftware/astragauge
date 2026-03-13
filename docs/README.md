# AstraGauge Documentation Index

## Purpose

This document is the **entry point** for the AstraGauge documentation set.

It provides:

- a map of the documentation tree
- recommended reading paths
- a quick explanation of what each document covers
- guidance on where new documentation should live

The goal is simple: keep the repo from turning into a markdown labyrinth with excellent intentions and terrible discoverability.

---

# Documentation Structure

```text
docs/
  README.md

  architecture/
    overview.md
    runtime.md
    panel-editor.md

  specs/
    panel-format.md
    sensor-schema.md
    sensor-store.md
    binding-engine.md
    widget-manifest.md
    provider-packaging.md
    theme-spec.md

  development/
    widget-guidelines.md
    provider-guidelines.md

  project/
    branding.md
    design-system.md
```

---

# Recommended Canonical Layout

## `docs/architecture/`

High-level system structure and major runtime/editor relationships.

### [`overview.md`](architecture/overview.md)
High-level map of the AstraGauge platform.

Use this when you want to understand:

- the major subsystems
- the core runtime data flow
- how the specs fit together

### [`runtime.md`](architecture/runtime.md)
Defines the internal runtime architecture.

Use this when working on:

- service boundaries
- runtime lifecycle
- update flow
- threading and batching
- panel session orchestration

### [`panel-editor.md`](architecture/panel-editor.md)
Defines the architecture of the visual panel editor.

Use this when working on:

- canvas interactions
- inspector design
- preview integration
- validation surfaces
- authoring workflows

---

## `docs/specs/`

Normative specifications for file formats, data models, and contracts between subsystems.

### `specs/panel-format.md`
Defines how panel documents are stored.

Use this when working on:

- panel serialization
- save/load
- portable panel artifacts
- widget placement structures

### `specs/sensor-schema.md`
Defines canonical sensor IDs and naming rules.

Use this when working on:

- provider normalization
- widget bindings
- portable panels
- cross-provider consistency

### `specs/sensor-store.md`
Defines the runtime sensor data model.

Use this when working on:

- latest-value storage
- history buffers
- subscriptions
- staleness handling
- data propagation

### `specs/binding-engine.md`
Defines how sensors bind to widget inputs.

Use this when working on:

- transforms
- aggregations
- wildcard resolution
- derived values
- widget input mapping

### `specs/widget-manifest.md`
Defines widget metadata and editor/runtime integration.

Use this when working on:

- widget discovery
- inspector generation
- binding targets
- widget sizing metadata
- widget validation

### `specs/provider-packaging.md`
Defines how providers are packaged, installed, and discovered.

Use this when working on:

- provider distribution
- manifest metadata
- compatibility checks
- plugin discovery

### `specs/theme-spec.md`
Defines the theme model.

Use this when working on:

- semantic color roles
- typography roles
- widget styling tokens
- theme loading

---

## `docs/development/`

Contributor-facing implementation guidance.

### `development/widget-guidelines.md`
Practical guidance for building widgets.

Use this when:

- implementing new widgets
- reviewing widget UX consistency
- validating widget behavior against the design system

### `development/provider-guidelines.md`
Practical guidance for building providers.

Use this when:

- implementing new providers
- reviewing provider lifecycle behavior
- normalizing sensor output correctly

---

## `docs/project/`

Project identity, product direction, and visual language.

### `project/branding.md`
Defines AstraGauge positioning, naming, and messaging.

Use this when working on:

- README copy
- project identity
- naming consistency
- public positioning

### `project/design-system.md`
Defines AstraGauge visual language and UI rules.

Use this when working on:

- widget appearance
- panel composition
- theming direction
- typography and spacing

---

# Recommended Reading Paths

## For new contributors

Read in this order:

1. [`architecture/overview.md`](architecture/overview.md)
2. [`project/branding.md`](project/branding.md)
3. [`project/design-system.md`](project/design-system.md)
4. [`architecture/runtime.md`](architecture/runtime.md)
5. [`specs/sensor-schema.md`](specs/sensor-schema.md)
6. [`specs/panel-format.md`](specs/panel-format.md)

This gives a clean overview before diving into subsystem details.

---

## For runtime work

Read in this order:

1. [`architecture/runtime.md`](architecture/runtime.md)
2. [`specs/sensor-store.md`](specs/sensor-store.md)
3. [`specs/binding-engine.md`](specs/binding-engine.md)
4. [`specs/provider-packaging.md`](specs/provider-packaging.md)
5. [`specs/sensor-schema.md`](specs/sensor-schema.md)

---

## For editor work

Read in this order:

1. `architecture/panel-editor.md`
2. `specs/panel-format.md`
3. `specs/widget-manifest.md`
4. `specs/binding-engine.md`
5. `project/design-system.md`

---

## For provider development

Read in this order:

1. `development/provider-guidelines.md`
2. `specs/sensor-schema.md`
3. `specs/provider-packaging.md`
4. `specs/sensor-store.md`

---

## For widget development

Read in this order:

1. `development/widget-guidelines.md`
2. `specs/widget-manifest.md`
3. `specs/binding-engine.md`
4. `project/design-system.md`
5. `specs/theme-spec.md`

---

# Documentation Rules

Use these conventions to keep the docs coherent.

## 1. Put normative contracts in `specs/`
If a document defines a format, schema, lifecycle contract, or cross-module rule, it belongs in `specs/`.

Examples:

- file formats
- manifest schemas
- canonical naming rules
- runtime data contracts

## 2. Put subsystem structure in `architecture/`
If a document explains how large parts of the system are organized, it belongs in `architecture/`.

Examples:

- runtime architecture
- editor architecture
- future multi-panel architecture

## 3. Put contributor advice in `development/`
If a document explains how to implement something well, it belongs in `development/`.

Examples:

- provider guidelines
- widget guidelines
- testing recommendations

## 4. Put identity and UX direction in `project/`
If a document explains what AstraGauge is, how it should look, or how it should be described, it belongs in `project/`.

Examples:

- branding
- design system
- vision
- product principles

---

# Suggested Future Documents

These are logical additions if the project grows.

## Likely future specs

```text
docs/specs/
  provider-api.md
  theme-manifest.md
  panel-template-format.md
  widget-packaging.md
  diagnostics-model.md
```

## Likely future architecture docs

```text
docs/architecture/
  multi-panel-runtime.md
  rendering-pipeline.md
  extension-loading.md
```

## Likely future project docs

```text
docs/project/
  vision.md
  roadmap.md
  ui-principles.md
```

---

# Suggested Root README References

The project root `README.md` should link to:

- `docs/README.md`
- `docs/architecture/overview.md`
- `docs/project/branding.md`

That gives new readers:

- a quick entry point
- architecture context
- product context

---

# Maintenance Guidance

To keep the documentation set healthy:

- update `docs/README.md` when new docs are added
- prefer editing existing specs over creating near-duplicates
- link related docs explicitly
- keep terminology consistent across all documents
- treat `specs/` as normative unless marked otherwise

If two docs disagree, the repo will eventually evolve into folklore. Folklore is charming in mythology, less so in software architecture.

---

# Summary

The AstraGauge docs should be navigated through four major areas:

- **architecture** — how the system is organized
- **specs** — the contracts that define the platform
- **development** — implementation guidance
- **project** — identity and design direction

Use this file as the stable entry point for the documentation tree.
