# AstraGauge Documentation Knowledge Base

**OVERVIEW**
Modular system instrumentation platform documentation.

**STRUCTURE**
- `architecture/`: High-level system structure and subsystem relationships.
- `specs/`: Normative specifications, data models, and cross-module contracts.
- `development/`: Implementation guidance and contributor-facing best practices.
- `project/`: Identity, branding, and visual language direction.

**WHERE TO LOOK**
- **System Map**: `architecture/overview.md` (subsystems and data flow).
- **Runtime Logic**: `architecture/runtime.md` (lifecycle, update flow).
- **Editor Logic**: `architecture/panel-editor.md` (canvas and inspector).
- **Sensor IDs**: `specs/sensor-schema.md` (canonical naming rules).
- **Panel Format**: `specs/panel-format.md` (serialization and placement).
- **Data Model**: `specs/sensor-store.md` (storage and subscriptions).
- **Binding Engine**: `specs/binding-engine.md` (sensor-to-widget mapping).
- **Widget Manifest**: `specs/widget-manifest.md` (metadata and integration).
- **Theme Model**: `specs/theme-spec.md` (semantic color/typography roles).
- **Widget Rules**: `development/widget-guidelines.md` (UX and behavior).
- **Provider Rules**: `development/provider-guidelines.md` (lifecycle and normalization).
- **UI Tokens**: `project/design-system.md` (semantic roles and grid).
- **Branding**: `project/branding.md` (positioning and messaging).

**CONVENTIONS**
- **Normative Contracts**: Place in `specs/` (formats, schemas, naming).
- **Subsystem Structure**: Place in `architecture/` (organization, flow).
- **Contributor Advice**: Place in `development/` (how-to, implementation).
- **Identity/UX**: Place in `project/` (branding, design principles).
- **Maintenance**: Update `docs/README.md` index when adding new files.
- **Consistency**: Prefer editing existing specs over creating duplicates.
- **Terminology**: Use canonical terms defined in `specs/sensor-schema.md`.

**NOTES**
- **Reading Paths**:
  - **New Contributors**: Overview -> Branding -> Design -> Runtime -> Schema -> Panel.
  - **Runtime Work**: Runtime -> Store -> Binding -> Packaging -> Schema.
  - **Widget Work**: Guidelines -> Manifest -> Binding -> Design -> Theme.
  - **Provider Work**: Guidelines -> Schema -> Packaging -> Store.
  - **Editor Work**: Panel-Editor -> Panel-Format -> Manifest -> Binding -> Design.
- **Future Docs**: See `docs/README.md` for planned additions.
- **Folklore Avoidance**: Treat `specs/` as the single source of truth.
