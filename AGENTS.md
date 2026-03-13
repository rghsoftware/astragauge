# AstraGauge Project Knowledge Base

**Generated:** 2026-03-12
**Commit:** af87c5b
**Branch:** main

---

## OVERVIEW

AstraGauge is a **modular system instrumentation platform** - desktop app for customizable system metrics dashboards. Built with Tauri v2 + SvelteKit 5 + Rust.

**Core Stack:** Tauri (desktop shell), SvelteKit (SPA frontend), Rust (backend/runtime), Bun (package manager)

**Architecture Pattern:** Providers → Sensor Store → Binding Engine → Widgets → Panels → Renderer

---

## STRUCTURE

```
astragauge/
├── apps/desktop/       # Tauri + SvelteKit desktop application
│   ├── src/            # SvelteKit frontend (routes, components)
│   └── src-tauri/      # Rust backend (Tauri commands)
├── crates/             # Rust workspace crates (EMPTY - reserved)
├── docs/               # Architecture specs & development guides
│   ├── architecture/   # Runtime, panel-editor, overview
│   ├── specs/          # Normative specs (sensor-schema, panel-format, etc.)
│   ├── development/    # Widget/provider guidelines
│   └── project/        # Branding, design-system
├── panels/             # Panel artifacts (EMPTY - reserved)
└── src/                # Placeholder Rust binary (not in workspace)
```

---

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| **Entry point (Rust)** | `apps/desktop/src-tauri/src/main.rs` | Calls `astragauge_lib::run()` |
| **Entry point (Frontend)** | `apps/desktop/src/routes/+page.svelte` | SvelteKit SPA entry |
| **Tauri commands** | `apps/desktop/src-tauri/src/lib.rs` | `#[tauri::command]` handlers |
| **Add Tauri command** | `lib.rs` → register in `generate_handler![]` | |
| **Frontend state** | `apps/desktop/src/routes/+layout.ts` | SSR disabled (`ssr = false`) |
| **Architecture specs** | `docs/architecture/` | runtime.md, panel-editor.md |
| **Data contracts** | `docs/specs/` | sensor-schema, panel-format, binding-engine |
| **Build config (Tauri)** | `apps/desktop/src-tauri/tauri.conf.json` | Port 1420, bun commands |
| **Build config (Vite)** | `apps/desktop/vite.config.js` | Fixed port, HMR config |

---

## CONVENTIONS

### Naming

**Sensor IDs** (`docs/specs/sensor-schema.md`):
```
Format: device.metric | device.component.metric
Rules: lowercase, dot-separated, singular device names, no units in ID
Examples: cpu.temperature, gpu.vram.used, cpu.core0.frequency
```

**Panel Files** (`docs/specs/panel-format.md`):
- Extension: `.panel.json`
- Grid-based: `{x, y, w, h}` coordinates

**Rust Library Naming**:
- Must use `_lib` suffix to avoid Windows conflicts: `astragauge_lib`

### Package Manager
**Bun** (not npm). Commands use `bun run`.

### SvelteKit SPA Mode
- `export const ssr = false` in `+layout.ts` (required for Tauri)
- Static adapter with `fallback: "index.html"`

### Design System (`docs/project/design-system.md`)
- **8px grid** for layout
- Monospace fonts for numeric values (JetBrains Mono, IBM Plex Mono)
- Semantic color roles: `theme.surface`, `theme.accent`, etc. (NEVER hardcode colors)

---

## ANTI-PATTERNS (THIS PROJECT)

### NEVER Crash the Runtime
- Providers, widgets, bindings should **fail locally, not catastrophically**
- Broken providers must not crash runtime
- Broken widgets must not break other widgets

### NEVER Bypass Theme System
- Use semantic roles, NOT hardcoded colors
- `docs/project/design-system.md` defines the token system

### NEVER Allow Direct Access
- Widgets MUST NOT access providers directly
- Widgets MUST NOT mutate store state
- Binding engine renders nothing - translation only

### NEVER Create Unbounded Growth
- Sensor buffers, history must have bounds
- Avoid unbounded memory accumulation

### NEVER Hardcode Logic
- Use schemas/manifests instead of hardcoded special cases
- Widget manifest drives inspector generation

### NEVER Mix Editor/Runtime Concerns
- Editor embeds runtime for preview, does not reinvent it
- Providers are runtime integrations, not editor plugins

---

## COMMANDS

```bash
# Development (from apps/desktop/)
bun run dev              # Vite dev server (port 1420)
bun run tauri dev        # Full Tauri dev mode (frontend + Rust)

# Build
bun run build            # SvelteKit production build
bun run tauri build      # Desktop app for distribution

# Type checking
bun run check            # One-time TypeScript/Svelte check
bun run check:watch      # Watch mode

# Rust (from apps/desktop/src-tauri/ or root)
cargo build              # Build Rust workspace
cargo run                # Run Tauri binary directly
```

---

## NOTES

### Known Issues

1. **Workspace Config Bug**: `Cargo.toml` declares `members = ["app/desktop", ...]` but actual path is `apps/desktop/` (plural). Fix: change to `apps/desktop`.

2. **Orphaned Root Binary**: `src/main.rs` at project root is not in workspace, contains only hello world. Consider removing or integrating.

3. **No Tests**: Project has no test infrastructure yet. Add before code grows.

4. **No CI/CD**: No GitHub Actions or automation configured.

### Documentation-First Development

Project follows docs-first approach - comprehensive specs exist before implementation. Key reading paths:

- **New contributors**: `docs/README.md` → `docs/architecture/overview.md` → `docs/project/branding.md`
- **Runtime work**: `docs/architecture/runtime.md` → `docs/specs/sensor-store.md`
- **Widget development**: `docs/development/widget-guidelines.md` → `docs/specs/widget-manifest.md`
- **Provider development**: `docs/development/provider-guidelines.md` → `docs/specs/sensor-schema.md`
