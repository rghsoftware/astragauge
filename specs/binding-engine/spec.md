# Binding Engine — Feature Spec

## User Stories

### US1: Widget Author — Unit Conversion
As a widget author, I want to convert sensor units (e.g., Celsius→Fahrenheit, bytes→GB) so my widgets display values in user-friendly units.

### US2: Widget Author — Chained Transforms
As a widget author, I want to apply multiple transforms in sequence (e.g., `percent` then `round(1)`) so I can compose data transformations without provider changes.

### US3: Runtime Engineer — Batch Resolution
As a runtime engineer, I want to resolve all panel bindings in a single batch call so I can minimize lock contention and reduce per-binding overhead.

### US4: Runtime Engineer — Pattern Caching
As a runtime engineer, I want wildcard patterns cached after first resolution so repeated wildcard bindings don't re-scan the sensor list.

### US5: Panel Editor — Binding Validation
As a panel editor, I want to validate a binding specification before registering it so I can provide immediate feedback on invalid bindings.

### US6: Widget Author — Value Formatting
As a widget author, I want resolved bindings to include formatted display strings so I don't need separate formatting logic in the widget.

## Functional Requirements

### FR1: Unit Conversion Transforms
Add transforms: `celsius_to_fahrenheit`, `bytes_to_kb`, `bytes_to_mb`, `bytes_to_gb`, `bytes_to_tb`, `bits_to_kbit`, `bits_to_mbit`, `bits_to_gbit`.

### FR2: Chained Transforms
Support pipe-delimited transform chains: `"percent|round(1)"`. Transforms apply left-to-right. `parse_transform` returns `Vec<Transform>` (chained).

### FR3: Batch Resolution
Add `resolve_batch(&self, bindings: &[Binding]) -> Vec<BindingResult<ResolvedBinding>>` method. Acquires store read lock once.

### FR4: Pattern Caching
Cache wildcard pattern → matched sensor IDs in `BindingEngine`. Cache invalidates when sensor list changes (use version counter from store or TTL).

### FR5: Binding Validation
Add `validate_binding(&self, binding: &Binding) -> BindingResult<()>` that checks: sensor exists (direct), pattern valid (wildcard), transform parseable.

### FR6: Value Formatting
Add `FormattedBinding` with `raw_value`, `formatted_value` (String), `unit`. Formatting rules: configurable decimal places, unit suffix.

## Success Criteria
1. All existing 90+ tests continue to pass
2. New unit tests for each FR (min 2 per FR)
3. Clippy clean
4. No breaking API changes — all new functionality is additive
