# Binding Engine — Tasks

## Task 1: Unit Conversion Transforms
Add celsius_to_fahrenheit, bytes_to_kb/mb/gb/tb, bits_to_kbit/mbit/gbit to Transform enum. Implement apply() for each. Add parse support. Tests.

## Task 2: Chained Transforms
Add `parse_transforms(s: &str) -> BindingResult<Vec<Transform>>` function. Pipe-delimited: `"percent|round(1)"`. Update BindingSubscription to use Vec<Transform>. Update resolve_with_transforms to accept &[Transform]. Backward compatible. Tests.

## Task 3: Batch Resolution
Add `resolve_batch(&self, bindings: &[Binding]) -> Vec<BindingResult<ResolvedBinding>>` to BindingEngine. Single lock acquisition. Tests.

## Task 4: Pattern Caching
Add PatternCache with TTL-based invalidation. Cache wildcard pattern → Vec<SensorId>. Default TTL 5s. Integration into resolve_wildcard. Tests.

## Task 5: Binding Validation
Add `validate_binding(&self, binding: &Binding) -> BindingResult<()>` to BindingEngine. Checks sensor exists, pattern valid, transform parseable. Tests.

## Task 6: Value Formatting
Add `format_value(resolved: &ResolvedBinding, spec: &FormatSpec) -> FormattedBinding` free function. FormatSpec: decimal_places, unit_suffix, na_string. Tests.

## Task 7: Integration Tests
Add integration tests covering: unit conversions, chained transforms, batch resolution, validation, formatting pipeline.

## Task 8: Verification
cargo test, clippy, full workspace build.
