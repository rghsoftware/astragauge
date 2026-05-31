use astragauge_domain::validation::SensorId;
use proptest::prelude::*;

prop_compose! {
  fn valid_segment()(s in "[a-z0-9]([a-z0-9-]*[a-z0-9])?") -> String {
    s
  }
}

prop_compose! {
  fn valid_sensor_id()(segments in prop::collection::vec(valid_segment(), 2..=4)) -> String {
    segments.join(".")
  }
}

proptest! {
  #[test]
  fn sensor_id_accepts_valid_patterns(id in valid_sensor_id()) {
    let result = SensorId::new(&id);
    prop_assert!(result.is_ok(), "expected '{}' to be valid, got err: {:?}", id, result.err());
  }

  #[test]
  fn sensor_id_roundtrips_through_str(id in valid_sensor_id()) {
    let sensor_id = SensorId::new(&id).unwrap();
    prop_assert_eq!(sensor_id.as_str(), id);
  }

  #[test]
  fn sensor_id_roundtrips_through_serde(id in valid_sensor_id()) {
    let sensor_id = SensorId::new(&id).unwrap();
    let json = serde_json::to_string(&sensor_id).unwrap();
    let parsed: SensorId = serde_json::from_str(&json).unwrap();
    prop_assert_eq!(sensor_id, parsed);
  }

  #[test]
  fn sensor_id_rejects_uppercase(s in "[A-Z]{2,10}") {
    let result = SensorId::new(&s);
    prop_assert!(result.is_err());
  }

  #[test]
  fn sensor_id_rejects_single_segment(s in "[a-z]{2,10}") {
    let result = SensorId::new(&s);
    prop_assert!(result.is_err());
  }

  #[test]
  fn sensor_id_rejects_unicode(s in "[\\p{Cyrillic}]{2,10}") {
    let result = SensorId::new(&s);
    prop_assert!(result.is_err());
  }
}

#[test]
fn sensor_id_numeric_only_segments() {
  assert!(SensorId::new("123.456").is_ok());
}

#[test]
fn sensor_id_hyphen_only_between_chars() {
  assert!(SensorId::new("a-b.c-d").is_ok());
}

#[test]
fn sensor_id_rejects_leading_hyphen_segment() {
  assert!(SensorId::new("-a.b").is_err());
}

#[test]
fn sensor_id_rejects_trailing_hyphen_segment() {
  assert!(SensorId::new("a-.b").is_err());
}

#[test]
fn sensor_id_allows_double_hyphen() {
  assert!(SensorId::new("a--b.c").is_ok());
}

#[test]
fn sensor_id_long_valid_id() {
  let id = "very-long-device-name-with-many-parts.component.metric-name";
  assert!(SensorId::new(id).is_ok());
}

#[test]
fn sensor_id_exactly_two_segments() {
  assert!(SensorId::new("a.b").is_ok());
}

#[test]
fn sensor_id_exactly_four_segments() {
  assert!(SensorId::new("a.b.c.d").is_ok());
}

#[test]
fn sensor_id_rejects_five_segments() {
  assert!(SensorId::new("a.b.c.d.e").is_err());
}

#[test]
fn sensor_id_single_char_segments() {
  assert!(SensorId::new("a.b").is_ok());
}
