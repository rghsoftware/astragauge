<!--
Sync Impact Report
==================
Version: 0.0.0 → 1.0.0 (MAJOR - initial ratification)
Modified principles: N/A (first ratification)
Added sections:
  - Core Principles (7 principles)
  - Technology & Design Constraints
  - Development Workflow
  - Governance
Removed sections: N/A
Templates requiring updates:
  - .specify/templates/plan-template.md ✅ compatible (Constitution Check section present)
  - .specify/templates/spec-template.md ✅ compatible (requirements structure aligns)
  - .specify/templates/tasks-template.md ✅ compatible (task categorization aligns)
  - .specify/templates/checklist-template.md ✅ compatible (no conflicts)
  - .specify/templates/commands/ ⚠ directory empty (no command files to update)
Follow-up TODOs: None
---
Version Change: 1.0.0 → 1.0.1 (PATCH - cleanup)
Modified principles: None
Added sections: None
Removed sections: Erroneous Tweeter Constitution (appended content, not part of AstraGauge)
Templates requiring updates:
  - .specify/templates/plan-template.md ✅ compatible
  - .specify/templates/spec-template.md ✅ compatible
  - .specify/templates/tasks-template.md ✅ compatible
  - .specify/templates/commands/ ⚠ directory empty
Follow-up TODOs: None
Amendment Rationale: Removed erroneously appended Tweeter Constitution content.
  No AstraGauge principles were modified.
-->

# AstraGauge Constitution

## Core Principles

### I. Fail Locally, Never Crash

Providers, widgets, bindings, and all runtime components MUST fail
gracefully within their own scope. A broken provider MUST NOT crash the
runtime. A broken widget MUST NOT break other widgets. Errors MUST be
contained, reported through structured channels, and recovered from
without cascading failure.

**Rationale**: AstraGauge is a persistent desktop instrument panel. Users
expect it to remain operational even when individual components
malfunction. Crash resilience is a foundational trust requirement.

### II. Theme-Driven Design

All visual output MUST use the semantic token system (`theme.surface`,
`theme.accent`, `theme.text-primary`, etc.). Hardcoded color values are
prohibited. Both light and dark modes MUST be supported from the start.
The design system defined in `docs/project/design-system.md` is the
authoritative source for tokens, typography, and spacing rules.

**Rationale**: A shared semantic token system ensures theme consistency,
enables user-customizable appearance, and prevents visual regressions
across independently developed widgets.

### III. Layered Architecture with Mediated Access

Widgets MUST NOT access providers directly. Widgets MUST NOT mutate
sensor store state. The binding engine translates and routes data — it
renders nothing itself. Data flows strictly through the pipeline:
Providers → Sensor Store → Binding Engine → Widgets. Every layer
communicates only through its defined interfaces.

**Rationale**: Strict layering prevents coupling, enables independent
testing and replacement of components, and ensures the runtime can
safely load and unload providers without affecting widgets.

### IV. Bounded Resources

Sensor buffers, history arrays, polling intervals, and all
memory-accumulating structures MUST have explicit upper bounds. No
component may grow unboundedly in memory or CPU usage. Bounds MUST be
configurable through the panel schema or provider manifest, never
hardcoded without override capability.

**Rationale**: AstraGauge runs as a long-lived desktop process.
Unbounded growth would eventually degrade performance or crash the
application, violating the instrument-panel reliability expectation.

### V. Schema-Driven Configuration

Widget behavior, provider capabilities, sensor metadata, and panel
layouts MUST be defined through schemas and manifests — not through
hardcoded logic or special-case conditionals. The inspector,
binding engine, and runtime MUST derive behavior from these declarations.

**Rationale**: Schema-driven configuration enables extensibility without
code changes, supports third-party widget/provider development, and
keeps the runtime generic rather than specialized per component.

### VI. Separation of Editor and Runtime Concerns

The editor embeds the runtime for preview purposes; it MUST NOT
reinvent runtime functionality. Providers are runtime integrations,
not editor plugins. Editor-specific state and UI concerns MUST be
isolated from runtime state and rendering.

**Rationale**: Mixing editor and runtime concerns creates dual-maintenance
burden, subtle bugs where preview behavior diverges from live behavior,
and architectural coupling that prevents independent evolution.

### VII. Documentation-First Development

Specifications and architecture documents MUST be written and reviewed
before implementation begins. New features MUST have corresponding
entries in `docs/specs/` and `docs/development/` before code is written.
Existing documentation MUST be updated when implementation diverges from
the documented design.

**Rationale**: AstraGauge's modular architecture (providers, widgets,
bindings) requires clear interface contracts. Documentation-first
development ensures contributors understand the system before modifying
it and prevents ad-hoc architectural decisions.

## Technology & Design Constraints

- **Stack**: Tauri v2 (desktop shell) + SvelteKit 5 (SPA frontend) +
  Rust (backend/runtime) + Bun (package manager)
- **SPA Mode**: SvelteKit MUST run with `export const ssr = false` in
  `+layout.ts` and use the static adapter with `fallback: "index.html"`
- **Package Manager**: Bun exclusively. All commands use `bun run`.
  Never use npm or yarn.
- **Rust Library Naming**: Tauri library crates MUST use the `_lib`
  suffix (e.g., `astragauge_lib`) to avoid conflicts on Windows.
- **8px Grid**: All widget layout and spacing MUST align to the 8px
  base grid defined in the design system.
- **Typography**: Monospaced fonts (JetBrains Mono, IBM Plex Mono) for
  numeric values to prevent jitter. Sans-serif (Inter, IBM Plex Sans)
  for labels and prose.
- **Sensor IDs**: Format `device.metric` or `device.component.metric`.
  Lowercase, dot-separated, singular device names, no units in ID.
- **Panel Files**: Extension `.panel.json`, grid-based with
  `{x, y, w, h}` coordinates.
- **Distance Legibility**: Primary goal is readability from secondary
  monitors at distance, prioritized over strict WCAG compliance where
  the two conflict (e.g., warning/critical state colors).

## Development Workflow

1. **Specification First**: Write spec in `docs/specs/` before code.
2. **Review**: Specs reviewed against constitution principles before
   implementation approval.
3. **Implement**: Follow layered architecture (III). Bound resources
   (IV). Schema-driven config (V).
4. **Test**: Write tests that validate component isolation (I),
   theme compliance (II), and interface contracts (III).
5. **Validate**: Run `bun run check` (TypeScript/Svelte) and
   `cargo build` / `cargo test` (Rust) before considering work complete.
6. **Document**: Update affected docs if implementation diverges from
   spec or design docs.

## Governance

This constitution is the authoritative governance document for the
AstraGauge project. It supersedes ad-hoc decisions, informal
conventions, and undocumented practices.

- **Amendments** require a written proposal documenting: the change,
  which principles are affected, migration plan for existing code, and
  rationale. Amendments increment the version per semantic versioning:
  MAJOR for principle removal/redefinition, MINOR for new/expanded
  principles, PATCH for clarifications.
- **Compliance**: All code changes MUST be verifiable against the
  principles above. Code review MUST check for principle violations.
- **Complexity Justification**: Any violation of the principles above
  MUST be documented in the relevant spec or plan with explicit
  rationale and a simpler rejected alternative.
- **Runtime Guidance**: Use `AGENTS.md` and `docs/architecture/` for
  implementation-level development guidance. This constitution provides
  the governing rules; those documents provide tactical direction.

**Version**: 1.0.1 | **Ratified**: 2026-05-30 | **Last Amended**: 2026-05-30
