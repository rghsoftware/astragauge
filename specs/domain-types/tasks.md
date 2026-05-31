# Tasks: Domain Types and Validation

**Feature**: domain-types
**Branch**: feature/time-machine-domain-types

## Tasks

### Task 1: Allow hyphens in SensorId segments
**File**: `crates/domain/src/validation.rs`
**What**: Update the character validation loop (line 53-63) to accept hyphens alongside lowercase letters and digits. Update error message. Change `test_invalid_hyphen` to `test_valid_hyphen` (expecting success).
**Test**: `cargo test -p astragauge-domain`

### Task 2: Add WidgetManifest::validate()
**File**: `crates/domain/src/widget.rs`
**What**: Add a `validate(&self) -> Result<(), DomainError>` method checking: non-empty id, name, category; version > 0; sizing consistency (default >= min, max >= default if set); required bindings satisfied. Add tests.
**Test**: `cargo test -p astragauge-domain`

### Task 3: Add proptest dev-dependency
**File**: `crates/domain/Cargo.toml`
**What**: Add `[dev-dependencies]` section with `proptest = "1"`
**Test**: `cargo check -p astragauge-domain`

### Task 4: Add property-based and edge-case tests
**File**: `crates/domain/tests/property_tests.rs` (new)
**What**: Property tests: SensorId accepts valid patterns (lowercase+digits+hyphens+dots, 2-4 segments), rejects invalid. Serialization roundtrips for all types. Edge cases: very long IDs, numeric-only segments, boundary segment counts.
**Test**: `cargo test -p astragauge-domain`

### Verification
- `cargo test -p astragauge-domain` — all tests pass
- `cargo clippy -p astragauge-domain` — no warnings
- `cargo build -p astragauge-domain` — clean build
