use astragauge_domain::SensorId;
use astragauge_sensor_store::{match_pattern, RingBuffer, SensorStore, StoreConfig};
use proptest::prelude::*;

mod common;

#[allow(unused_imports)]
use common::{make_descriptor, make_sample};

proptest! {
  #[test]
  fn history_never_exceeds_capacity(pushes in 0..500usize, capacity in 1..200usize) {
    let mut buffer = RingBuffer::new(capacity);

    for i in 0..pushes {
      buffer.push(i);
    }

    prop_assert!(buffer.len() <= capacity);
  }
}

proptest! {
  #[test]
  fn history_retains_most_recent_items(pushes in 1..500usize, capacity in 1..100usize) {
    let mut buffer = RingBuffer::new(capacity);

    for i in 0..pushes {
      buffer.push(i);
    }

    let items: Vec<_> = buffer.iter().copied().collect();
    let expected_len = pushes.min(capacity);

    prop_assert_eq!(items.len(), expected_len);

    if pushes > capacity {
      let expected_first = pushes - capacity;
      prop_assert_eq!(items[0], expected_first);
      prop_assert_eq!(items[capacity - 1], pushes - 1);
    } else {
      prop_assert_eq!(items[0], 0);
      prop_assert_eq!(items[pushes - 1], pushes - 1);
    }
  }
}

proptest! {
  #[test]
  fn pattern_matching_deterministic(
    first_segment in "[a-z]{2,4}",
    second_segment in "[a-z0-9]{1,4}",
    third_segment in "[a-z]{3,6}"
  ) {
    let id1 = SensorId::new(format!("{}.{}.{}", first_segment, second_segment, third_segment)).unwrap();
    let id2 = SensorId::new(format!("{}.other.{}", first_segment, third_segment)).unwrap();
    let id3 = SensorId::new(format!("{}.{}.{}", first_segment, "core1", third_segment)).unwrap();
    let ids = vec![id1.clone(), id2.clone(), id3.clone()];

    let pattern = format!("{}.*.{}", first_segment, third_segment);

    let result1 = match_pattern(&pattern, &ids);
    let result2 = match_pattern(&pattern, &ids);
    let result3 = match_pattern(&pattern, &ids);

    prop_assert_eq!(result1, result2.clone());
    prop_assert_eq!(result2, result3);
  }
}

proptest! {
  #[test]
  fn pattern_matching_consistent_with_wildcard(
    device in "[a-z]{2,4}",
    metric in "[a-z]{3,6}"
  ) {
    let ids = vec![
      SensorId::new(format!("{}.temperature", device)).unwrap(),
      SensorId::new(format!("{}.frequency", device)).unwrap(),
      SensorId::new(format!("{}.utilization", device)).unwrap(),
      SensorId::new(format!("other.{}", metric)).unwrap(),
    ];

    let wildcard_pattern = format!("{}.*", device);
    let result = match_pattern(&wildcard_pattern, &ids);
    let result2 = match_pattern(&wildcard_pattern, &ids);

    prop_assert_eq!(result.clone(), result2);

    let prefix = format!("{}.", device);
    for id in &result {
      prop_assert!(id.as_str().starts_with(&prefix));
    }
  }
}

proptest! {
  #[test]
  fn staleness_threshold_boundary(
    staleness_threshold_ms in 100u64..10000u64,
    sample_age_delta in -500i64..500i64
  ) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
      let config = StoreConfig::new()
        .with_history_capacity(120)
        .with_staleness_threshold_ms(staleness_threshold_ms);
      let store = SensorStore::with_config(config);
      let id = SensorId::new("cpu.temperature").unwrap();

      store
        .register_sensor(make_descriptor("cpu.temperature"))
        .await
        .unwrap();

      let sample_time = 10000u64;
      let now = (sample_time as i64 + sample_age_delta).max(0) as u64;

      store
        .push_sample(make_sample("cpu.temperature", sample_time, Some(72.5)))
        .await
        .unwrap();

      let is_stale = store.is_stale(&id, now).await;
      let age = now.saturating_sub(sample_time);

      if age > staleness_threshold_ms {
        prop_assert!(is_stale);
      } else {
        prop_assert!(!is_stale);
      }
      Ok(())
    }).unwrap();
  }
}

proptest! {
  #[test]
  fn fresh_sample_never_stale(sample_time in 0u64..1000000u64) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
      let store = SensorStore::new();
      let id = SensorId::new("cpu.temperature").unwrap();

      store
        .register_sensor(make_descriptor("cpu.temperature"))
        .await
        .unwrap();

      store
        .push_sample(make_sample("cpu.temperature", sample_time, Some(72.5)))
        .await
        .unwrap();

      let is_stale = store.is_stale(&id, sample_time).await;
      prop_assert!(!is_stale);
      Ok(())
    }).unwrap();
  }
}

proptest! {
  #[test]
  fn old_sample_always_stale(age_beyond_threshold in 1u64..100000u64) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
      let staleness_threshold_ms = 5000u64;
      let config = StoreConfig::new()
        .with_history_capacity(120)
        .with_staleness_threshold_ms(staleness_threshold_ms);
      let store = SensorStore::with_config(config);
      let id = SensorId::new("cpu.temperature").unwrap();

      store
        .register_sensor(make_descriptor("cpu.temperature"))
        .await
        .unwrap();

      let sample_time = 0u64;
      let now = staleness_threshold_ms + age_beyond_threshold;

      store
        .push_sample(make_sample("cpu.temperature", sample_time, Some(72.5)))
        .await
        .unwrap();

      let is_stale = store.is_stale(&id, now).await;
      prop_assert!(is_stale);
      Ok(())
    }).unwrap();
  }
}
