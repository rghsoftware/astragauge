# AstraGauge

**Instrument your system. Style it your way.**

AstraGauge is a modular system instrumentation platform for building customizable dashboards that display hardware and system metrics.

## Features

- **Modular Architecture** — Providers collect metrics, widgets render them, panels compose layouts
- **Cross-Platform** — Built with Tauri for Windows, macOS, and Linux
- **Customizable Panels** — Create and arrange sensor widgets to match your workflow
- **Themeable** — Apply visual themes to personalize your dashboard
- **Extensible** — Build custom providers and widgets

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Shell | Tauri v2 |
| Frontend | SvelteKit 5 |
| Backend | Rust |
| Package Manager | Bun |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Bun](https://bun.sh/) (v1.0+)
- [Node.js](https://nodejs.org/) (optional, for tooling)

### Development

```bash
# Clone the repository
git clone https://github.com/rghsoftware/astragauge.git
cd astragauge

# Install frontend dependencies
cd apps/desktop
bun install

# Start development mode (frontend + Rust)
bun run tauri dev
```

### Build

```bash
# Build frontend
bun run build

# Build desktop application
bun run tauri build
```

## Project Structure

```
astragauge/
├── apps/desktop/       # Tauri + SvelteKit desktop application
│   ├── src/            # SvelteKit frontend
│   └── src-tauri/      # Rust backend
├── docs/               # Architecture specs & development guides
├── crates/             # Rust workspace crates (reserved)
└── panels/             # Panel artifacts (reserved)
```

## Documentation

| Document | Description |
|----------|-------------|
| [Documentation Index](docs/README.md) | Entry point for all documentation |
| [Architecture Overview](docs/architecture/overview.md) | High-level system map |
| [Branding Guide](docs/project/branding.md) | Project identity and positioning |

### Reading Paths

**New contributors:** Start with [Architecture Overview](docs/architecture/overview.md) → [Branding](docs/project/branding.md) → [Design System](docs/project/design-system.md)

**Runtime work:** [Runtime Architecture](docs/architecture/runtime.md) → [Sensor Store](docs/specs/sensor-store.md) → [Binding Engine](docs/specs/binding-engine.md)

**Widget development:** [Widget Guidelines](docs/development/widget-guidelines.md) → [Widget Manifest](docs/specs/widget-manifest.md) → [Design System](docs/project/design-system.md)

## Contributing

We welcome contributions! Please read the documentation before submitting PRs.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `bun run check` and `cargo clippy` to verify
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.
