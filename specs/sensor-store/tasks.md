# Tasks: sensor-store

## Task 1: Integrate SubscriptionManager into SensorStore
- Add `SubscriptionManager` field to `SensorStoreInner`
- Expose `subscribe(pattern)` and `unsubscribe(id)` on `SensorStore`
- Ensure SubscriptionManager is initialized in `new()` and `with_config()`

## Task 2: Add notification to push_sample
- After pushing a sample, call `SubscriptionManager::notify_matching` 
- Must not hold write lock while notifying (use clone-then-notify pattern to avoid deadlock)

## Task 3: Add batch notification to push_samples
- Collect all samples, then notify per unique pattern once
- Again, avoid holding write lock during notification

## Task 4: Add list_sensors_by_category
- Read-lock the store, filter descriptors by category, return matching SensorIds

## Task 5: Add query_pattern
- Read-lock the store, use existing `match_pattern` against registered sensor IDs

## Task 6: Add get_stale_sensors
- Read-lock the store, check each sensor's last_update against now_ms + threshold

## Task 7: Add sensor_count
- Read-lock, return descriptors.len()

## Task 8: Add integration tests
- Test subscribe → push → receive
- Test batch notification deduplication
- Test list_sensors_by_category
- Test query_pattern
- Test get_stale_sensors

## Task 9: Verification
- cargo test, clippy, build
