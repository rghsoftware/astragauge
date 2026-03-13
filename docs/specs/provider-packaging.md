# Provider Packaging Specification

Status: Draft  
Version: 0.1  
Owner: Providers  
Last Updated: 2026-03-12

---

# 1. Purpose

This document defines how **AstraGauge Providers** are packaged,
distributed, discovered, and loaded by the runtime.

Providers are responsible for:

-   discovering sensors
-   polling sensor values
-   normalizing sensor descriptors and samples

The packaging format ensures that providers can be:

-   distributed independently of the runtime
-   versioned safely
-   discovered automatically
-   validated before loading

------------------------------------------------------------------------

# 2. Goals

The provider packaging system must support:

-   **runtime discovery**
-   **clear provider metadata**
-   **safe version compatibility checks**
-   **portable distribution**
-   **future ecosystem extensibility**

Providers should be installable without modifying the core runtime.

------------------------------------------------------------------------

# 3. Non‑Goals

This specification does not define:

-   the internal provider implementation API
-   sensor schema rules
-   panel behavior
-   widget rendering
-   runtime plugin sandboxing

Those are handled by other specifications.

------------------------------------------------------------------------

# 4. Provider Package Overview

A provider package contains:

-   provider manifest
-   compiled implementation
-   optional assets
-   optional documentation

Example structure:

    provider/
      provider.toml
      provider.bundle
      README.md

The **manifest** describes the provider while the bundle contains the
compiled implementation.

------------------------------------------------------------------------

# 5. Provider Manifest

Each provider must include a manifest file.

Recommended format:

    provider.toml

Example:

``` toml
id = "core.linux"
name = "Linux System Provider"
version = "0.1.0"
description = "Collects system metrics from Linux using /proc and hwmon."
author = "AstraGauge Project"
runtime = ">=0.1.0"

[sensors]
categories = ["cpu", "memory", "disk", "network", "temperature"]
```

------------------------------------------------------------------------

# 6. Required Manifest Fields

  Field         Description
  ------------- ---------------------------------
  id            stable provider identifier
  name          human‑readable name
  version       provider version
  description   short summary
  runtime       supported runtime version range

Provider IDs should follow:

    namespace.provider

Examples:

    core.linux
    core.windows
    community.nvidia
    community.openhardware

------------------------------------------------------------------------

# 7. Optional Manifest Fields

  Field        Description
  ------------ ------------------------------
  author       provider maintainer
  website      project homepage
  repository   source repository
  license      provider license
  tags         optional categorization tags

Example:

``` toml
author = "Example Maintainer"
license = "MIT"
tags = ["gpu", "hardware"]
```

------------------------------------------------------------------------

# 8. Provider Capabilities

Providers may advertise capabilities so the runtime and editor can
present useful information.

Example:

``` toml
[capabilities]
historical = false
high_frequency = true
hardware_access = true
```

Suggested capability flags:

  Flag              Meaning
  ----------------- ----------------------------------------
  historical        provider exposes historical data
  high_frequency    supports high update rates
  hardware_access   accesses low‑level hardware interfaces

------------------------------------------------------------------------

# 9. Sensor Categories

Providers should declare which sensor categories they support.

Example:

``` toml
[sensors]
categories = ["cpu", "gpu", "memory"]
```

Categories are informational and help the editor filter sensors.

------------------------------------------------------------------------

# 10. Packaging Format

A provider distribution should be packaged as a compressed archive.

Recommended extension:

    .provider

Example:

    linux.provider

Archive contents:

    provider.toml
    provider.bundle
    README.md

The runtime extracts and registers the provider.

------------------------------------------------------------------------

# 11. Installation Locations

Providers should be discoverable from standard directories.

Example search paths:

    ~/.astragauge/providers
    /usr/share/astragauge/providers
    ./providers

The runtime scans these directories on startup.

------------------------------------------------------------------------

# 12. Provider Loading

Provider loading flow:

1.  runtime scans provider directories
2.  provider manifests are parsed
3.  version compatibility is validated
4.  provider bundle is loaded
5.  provider initialization runs
6.  sensors are registered

Example:

    Provider Archive → Manifest Parse → Compatibility Check → Load → Initialize

------------------------------------------------------------------------

# 13. Version Compatibility

Providers must specify runtime compatibility.

Example:

    runtime = ">=0.1.0"

The runtime should reject providers outside the supported range.

This prevents crashes caused by API mismatches.

------------------------------------------------------------------------

# 14. Security Considerations

Providers execute code inside the runtime process.

Recommendations:

-   validate manifests before loading
-   isolate provider errors
-   avoid crashing the runtime
-   consider sandboxing in the future

Initial versions may rely on trust and signed distributions.

------------------------------------------------------------------------

# 15. Provider Discovery

The runtime should expose provider metadata to the editor.

Example information:

-   provider name
-   supported sensor categories
-   health status
-   available sensors

This enables better binding workflows.

------------------------------------------------------------------------

# 16. Provider Health Reporting

Providers should expose a health state.

Example:

  State      Meaning
  ---------- -------------------------------
  ok         provider functioning normally
  degraded   partial sensor failure
  error      provider unable to function

The runtime should surface this in the UI.

------------------------------------------------------------------------

# 17. Example Provider Package

Example directory:

    linux.provider/
      provider.toml
      provider.bundle
      README.md

Example manifest:

``` toml
id = "core.linux"
name = "Linux System Provider"
version = "0.1.0"
runtime = ">=0.1.0"

[sensors]
categories = ["cpu","memory","disk","network","temperature"]
```

------------------------------------------------------------------------

# 18. Recommended Initial Providers

The first official providers may include:

-   `core.linux`
-   `core.windows`
-   `core.mac`
-   `community.openhardware`
-   `community.nvidia`

These cover most common sensor sources.

------------------------------------------------------------------------

# 19. Future Extensions

Possible future enhancements:

-   signed provider packages
-   provider sandboxing
-   provider capability negotiation
-   dependency declarations
-   remote provider repositories
-   automatic updates

These should be added only after the basic ecosystem is stable.

------------------------------------------------------------------------

# 20. Summary

The AstraGauge Provider Packaging Specification defines a simple system
for distributing and loading sensor providers.

The packaging model enables:

-   modular sensor integrations
-   runtime discovery
-   version safety
-   ecosystem growth

A well‑defined provider packaging format prevents the platform from
devolving into plugin chaos.
