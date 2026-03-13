use astragauge_domain::SensorId;

/// Matches a wildcard pattern against sensor IDs.
///
/// Pattern format: `segment.segment.segment` where each segment can be:
/// - A literal string
/// - `*` (full segment wildcard - matches any single segment)
/// - Contains `*` as part of the segment (partial wildcard)
///
/// # Examples
///
/// Full segment wildcard:
/// ```ignore
/// use astragauge_sensor_store::match_pattern;
/// use astragauge_domain::SensorId;
///
/// let ids = vec![
///     SensorId::new("cpu.core0.temperature").unwrap(),
///     SensorId::new("cpu.core1.temperature").unwrap(),
///     SensorId::new("gpu.temperature").unwrap(),
/// ];
/// let result = match_pattern("cpu.*.temperature", &ids);
/// // Matches cpu.core0.temperature and cpu.core1.temperature
/// ```
///
/// Partial segment wildcard:
/// ```ignore
/// use astragauge_sensor_store::match_pattern;
/// use astragauge_domain::SensorId;
///
/// let ids = vec![
///     SensorId::new("cpu.core0.temperature").unwrap(),
///     SensorId::new("cpu.core1.temperature").unwrap(),
/// ];
/// let result = match_pattern("cpu.core*.temperature", &ids);
/// // Matches both: core0 and core1 match core*
/// ```
///
/// # Arguments
///
/// * `pattern` - The wildcard pattern to match
/// * `ids` - Slice of SensorId values to test against
///
/// # Returns
///
/// Vector of matching SensorIds in the same order as input
pub fn match_pattern(pattern: &str, ids: &[SensorId]) -> Vec<SensorId> {
  if pattern.is_empty() {
    return Vec::new();
  }

  let pattern_segments: Vec<&str> = pattern.split('.').collect();

  ids
    .iter()
    .filter(|id| {
      let id_str = id.as_str();
      let id_segments: Vec<&str> = id_str.split('.').collect();

      if pattern_segments.len() != id_segments.len() {
        return false;
      }

      pattern_segments
        .iter()
        .zip(id_segments.iter())
        .all(|(p, s)| match_segment(p, s))
    })
    .cloned()
    .collect()
}

/// Matches a single pattern segment against an ID segment.
///
/// Supports:
/// - `*` (full wildcard): matches any segment
/// - `prefix*` (partial wildcard): matches segments starting with prefix
/// - `*suffix` (partial wildcard): matches segments ending with suffix
/// - `prefix*suffix` (partial wildcard): matches segments with prefix and suffix
/// - literal: exact match required
fn match_segment(pattern: &str, segment: &str) -> bool {
  if pattern == "*" {
    true
  } else if pattern.contains('*') {
    // Split on '*' - this can produce 1, 2, or more parts
    // 1 part: pattern was "*" (already handled above) or empty (invalid)
    // 2 parts: prefix*suffix patterns (including empty prefix/suffix)
    // 3+ parts: multiple wildcards in segment - not supported
    let (prefix, suffix) = pattern.split_once('*').unwrap_or(("", ""));

    // If there are more wildcards, not supported
    if suffix.contains('*') {
      return false;
    }

    segment.starts_with(prefix) && segment.ends_with(suffix)
  } else {
    pattern == segment
  }
}

/// Checks if a single sensor ID matches the pattern.
///
/// This is an allocation-free alternative to [`match_pattern`] when checking
/// a single ID against a pattern, avoiding the Vec allocation.
///
/// # Arguments
///
/// * `pattern` - The wildcard pattern to match
/// * `sensor_id` - The sensor ID to test
///
/// # Returns
///
/// `true` if the sensor ID matches the pattern
pub fn matches_single(pattern: &str, sensor_id: &SensorId) -> bool {
  if pattern.is_empty() {
    return false;
  }

  let id_str = sensor_id.as_str();
  let pattern_segments: Vec<&str> = pattern.split('.').collect();
  let id_segments: Vec<&str> = id_str.split('.').collect();

  if pattern_segments.len() != id_segments.len() {
    return false;
  }

  pattern_segments
    .iter()
    .zip(id_segments.iter())
    .all(|(p, s)| match_segment(p, s))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_id(s: &str) -> SensorId {
    SensorId::new(s).unwrap()
  }

  #[test]
  fn test_exact_match() {
    let ids = vec![make_id("cpu.temperature")];
    let result = match_pattern("cpu.temperature", &ids);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].as_str(), "cpu.temperature");
  }

  #[test]
  fn test_wildcard_single_segment() {
    let ids = vec![
      make_id("cpu.core0.temperature"),
      make_id("cpu.core1.temperature"),
      make_id("gpu.temperature"),
    ];
    let result = match_pattern("cpu.*.temperature", &ids);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].as_str(), "cpu.core0.temperature");
    assert_eq!(result[1].as_str(), "cpu.core1.temperature");
  }

  #[test]
  fn test_different_segment_count() {
    let ids = vec![make_id("cpu.temperature"), make_id("cpu.core0.temperature")];
    let result = match_pattern("cpu.*", &ids);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].as_str(), "cpu.temperature");
  }

  #[test]
  fn test_partial_wildcard_prefix() {
    let ids = vec![
      make_id("cpu.core0.temperature"),
      make_id("cpu.core1.temperature"),
      make_id("cpu.core15.temperature"),
      make_id("gpu.temperature"),
    ];
    let result = match_pattern("cpu.core*.temperature", &ids);
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|id| id.as_str().starts_with("cpu.core")));
  }

  #[test]
  fn test_partial_wildcard_suffix() {
    let ids = vec![
      make_id("cpu.temp0"),
      make_id("cpu.temp1"),
      make_id("cpu.temperature"),
    ];
    let result = match_pattern("cpu.*temp", &ids);
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_no_match() {
    let ids = vec![make_id("cpu.temperature")];
    let result = match_pattern("gpu.temperature", &ids);
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_empty_pattern() {
    let ids = vec![make_id("cpu.temperature")];
    let result = match_pattern("", &ids);
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_wildcard_all_segments() {
    let ids = vec![
      make_id("cpu.temperature"),
      make_id("gpu.temperature"),
      make_id("memory.used"),
    ];
    let result = match_pattern("*.*", &ids);
    assert_eq!(result.len(), 3);
  }

  #[test]
  fn test_multiple_patterns() {
    let ids = vec![
      make_id("cpu.core0.temperature"),
      make_id("cpu.core1.temperature"),
      make_id("gpu.core0.temperature"),
      make_id("cpu.core0.frequency"),
    ];

    let result = match_pattern("cpu.core*.temperature", &ids);
    assert_eq!(result.len(), 2);
    assert!(result
      .iter()
      .all(|id| id.as_str().starts_with("cpu.core") && id.as_str().ends_with(".temperature")));
  }

  #[test]
  fn test_empty_ids() {
    let ids: Vec<SensorId> = vec![];
    let result = match_pattern("cpu.*", &ids);
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_preserves_order() {
    let ids = vec![
      make_id("cpu.core1.temperature"),
      make_id("cpu.core0.temperature"),
      make_id("cpu.core2.temperature"),
    ];
    let result = match_pattern("cpu.core*.temperature", &ids);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].as_str(), "cpu.core1.temperature");
    assert_eq!(result[1].as_str(), "cpu.core0.temperature");
    assert_eq!(result[2].as_str(), "cpu.core2.temperature");
  }

  #[test]
  fn test_matches_single_exact() {
    let id = make_id("cpu.temperature");
    assert!(matches_single("cpu.temperature", &id));
    assert!(!matches_single("gpu.temperature", &id));
  }

  #[test]
  fn test_matches_single_wildcard() {
    let id = make_id("cpu.core0.temperature");
    assert!(matches_single("cpu.*.temperature", &id));
    assert!(matches_single("cpu.core*.temperature", &id));
    assert!(!matches_single("gpu.*.temperature", &id));
  }

  #[test]
  fn test_matches_single_empty_pattern() {
    let id = make_id("cpu.temperature");
    assert!(!matches_single("", &id));
  }

  #[test]
  fn test_matches_single_different_segment_count() {
    let id = make_id("cpu.temperature");
    assert!(!matches_single("cpu.*.temperature", &id));
  }
}
