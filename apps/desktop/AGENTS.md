# AstraGauge Desktop App Knowledge Base

**Path:** `apps/desktop/`
**Stack:** Tauri v2, SvelteKit 5, Rust, Bun

---

## OVERVIEW
Tauri v2 + SvelteKit 5 desktop application shell and frontend implementation.

---

## STRUCTURE
```
apps/desktop/
├── src/                # SvelteKit frontend
│   ├── lib/            # Shared frontend components/utils
│   └── routes/         # File-based routing (+page.svelte, +layout.ts)
├── src-tauri/          # Rust backend (Tauri Core)
│   ├── src/            # Rust source (main.rs, lib.rs)
│   ├── capabilities/   # Tauri v2 permission sets
│   └── tauri.conf.json # App configuration
├── static/             # Static frontend assets
├── svelte.config.js    # SvelteKit configuration (SPA mode)
└── vite.config.js      # Vite configuration (Tauri integration)
```

---

## WHERE TO LOOK

| Task | Location |
|------|----------|
| **Frontend Entry** | `src/routes/+page.svelte` |
| **Global Layout** | `src/routes/+layout.svelte` |
| **SSR Configuration** | `src/routes/+layout.ts` |
| **Rust Commands** | `src-tauri/src/lib.rs` |
| **App Permissions** | `src-tauri/capabilities/` |
| **Build Output** | `build/` (generated) |
| **Tauri Config** | `src-tauri/tauri.conf.json` |

---

## CONVENTIONS

### SvelteKit Integration
- **SPA Mode Only**: `export const ssr = false` in `+layout.ts` is mandatory.
- **Static Adapter**: Uses `@sveltejs/adapter-static` with `fallback: "index.html"`.
- **Routing**: Standard SvelteKit file-based routing.

### Tauri IPC
- **Invoke**: Use `import { invoke } from "@tauri-apps/api/core"` for Rust calls.
- **Commands**: Define in `lib.rs` with `#[tauri::command]`, register in `generate_handler!`.
- **Events**: Use `@tauri-apps/api/event` for fire-and-forget notifications.

### Frontend Patterns
- **Reactivity**: Use Svelte 5 runes (`$state`, `$derived`, `$effect`).
- **Styling**: Scoped `<style>` blocks or global CSS in `src/app.css`.
- **Strict Port**: Vite must use port 1420 (`strictPort: true`) for Tauri IPC.

---

## COMMANDS

```bash
# From apps/desktop/
bun run dev              # Start Vite dev server (port 1420)
bun run tauri dev        # Start full Tauri dev mode (Frontend + Rust)
bun run build            # Production frontend build
bun run tauri build      # Create production desktop installers
bun run check            # Svelte/TypeScript type check
```

---

## NOTES

- **Windows Compatibility**: Rust library named `astragauge_lib` to avoid reserved name conflicts.
- **HMR**: Configured in `vite.config.js` to work with Tauri's webview.
- **Security**: CSP is managed in `tauri.conf.json` under `app.security.csp`.
- **Permissions**: Tauri v2 uses a capability-based security model in `src-tauri/capabilities/`.
- **Dist**: Frontend build output goes to `../build` relative to `src-tauri`.
