# Tasks: provider-host

## Task 1: Add manifest validation to register_provider
- Call `manifest.validate()` during registration
- Return `ProviderError::InvalidManifest` on failure

## Task 2: Add unregister_provider
- Cancel the provider's task via shutdown token (per-provider tokens needed)
- Wait for task completion with timeout
- Remove from providers map

## Task 3: Add get_provider_health(id)
- Return `Option<ProviderHealth>` for a single provider

## Task 4: Add is_provider_running(id)
- Return true if the provider has an active task

## Task 5: Fix ignored panic containment tests
- Remove `#[ignore]` attribute from the 3 unit tests
- Ensure they pass reliably

## Task 6: Add integration tests
- Test manifest validation rejection
- Test unregister_provider stops tasks
- Test get_provider_health
- Test is_provider_running

## Task 7: Verification
- cargo test, clippy, build
