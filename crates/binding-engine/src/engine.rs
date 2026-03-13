use astragauge_domain::SensorSample;
use astragauge_sensor_store::{pattern::match_pattern, SensorStore};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{
  Aggregation, Binding, BindingError, BindingResult, BindingSource, ResolvedBinding, Transform,
};

/// Core engine for resolving sensor bindings.
///
/// The `BindingEngine` translates abstract binding specifications into concrete
/// resolved values by:
/// 1. Looking up sensor values from the `SensorStore`
/// 2. Applying wildcard pattern matching for multi-sensor bindings
/// 3. Aggregating multiple values when needed
/// 4. Applying transformations to the final result
#[derive(Clone)]
pub struct BindingEngine {
  store: Arc<RwLock<SensorStore>>,
}

impl BindingEngine {
  /// Creates a new `BindingEngine` with the given sensor store.
  #[must_use]
  pub fn new(store: SensorStore) -> Self {
    Self {
      store: Arc::new(RwLock::new(store)),
    }
  }

  /// Creates a `BindingEngine` from an existing `Arc<RwLock<SensorStore>>`.
  #[must_use]
  pub fn from_shared(store: Arc<RwLock<SensorStore>>) -> Self {
    Self { store }
  }

  /// Resolves a binding to its final value.
  ///
  /// # Arguments
  ///
  /// * `binding` - The binding specification to resolve
  ///
  /// # Returns
  ///
  /// A `ResolvedBinding` containing the final value and source count,
  /// or a `BindingError` if resolution fails.
  pub async fn resolve(&self, binding: &Binding) -> BindingResult<ResolvedBinding> {
    let (value, source_count) = match &binding.source {
      BindingSource::Direct { sensor_id } => self.resolve_direct(sensor_id).await?,
      BindingSource::Wildcard {
        pattern,
        aggregation,
      } => self.resolve_wildcard(pattern, aggregation).await?,
    };

    let final_value = if let Some(transform_str) = &binding.transform {
      let transform = parse_transform(transform_str)?;
      transform.apply(value)
    } else {
      value
    };

    Ok(ResolvedBinding {
      value: final_value,
      source_count,
    })
  }

  /// Resolves a binding with a pre-parsed transform for better performance.
  ///
  /// Use this method when the transform has already been parsed to avoid
  /// redundant parsing on high-frequency updates.
  pub async fn resolve_with_transform(
    &self,
    binding: &Binding,
    transform: Option<&Transform>,
  ) -> BindingResult<ResolvedBinding> {
    let (value, source_count) = match &binding.source {
      BindingSource::Direct { sensor_id } => self.resolve_direct(sensor_id).await?,
      BindingSource::Wildcard {
        pattern,
        aggregation,
      } => self.resolve_wildcard(pattern, aggregation).await?,
    };

    let final_value = match transform {
      Some(t) => t.apply(value),
      None => value,
    };

    Ok(ResolvedBinding {
      value: final_value,
      source_count,
    })
  }

  async fn resolve_direct(
    &self,
    sensor_id: &astragauge_domain::SensorId,
  ) -> BindingResult<(Option<f64>, usize)> {
    let store = self.store.read().await;
    let sample: Option<SensorSample> = store.get_value(sensor_id).await;

    match sample {
      Some(s) => Ok((s.value, 1)),
      None => Err(BindingError::UnresolvedSensor(
        sensor_id.as_str().to_string(),
      )),
    }
  }

  async fn resolve_wildcard(
    &self,
    pattern: &str,
    aggregation: &Aggregation,
  ) -> BindingResult<(Option<f64>, usize)> {
    let store = self.store.read().await;
    let all_sensors = store.list_sensors().await;

    let matching_ids = match_pattern(pattern, &all_sensors);

    if matching_ids.is_empty() {
      return Err(BindingError::WildcardNoMatch(pattern.to_string()));
    }

    let mut values = Vec::with_capacity(matching_ids.len());
    for id in &matching_ids {
      let sample = store.get_value(id).await;
      values.push(sample.and_then(|s| s.value));
    }

    let aggregated = aggregation.apply(&values);
    let source_count = matching_ids.len();

    Ok((aggregated, source_count))
  }
}

/// Parses a transform string into a `Transform` enum.
///
/// Supported formats:
/// - `"round(N)"` - Round to N decimal places
/// - `"clamp(min,max)"` - Clamp to range [min, max]
/// - `"abs"` - Absolute value
/// - `"scale(factor)"` - Scale by factor
/// - `"percent"` - Multiply by 100
pub fn parse_transform(s: &str) -> BindingResult<Transform> {
  let s = s.trim();

  if s == "abs" {
    return Ok(Transform::Abs);
  }

  if s == "percent" {
    return Ok(Transform::Percent);
  }

  if let Some(rest) = s.strip_prefix("round(") {
    if let Some(digits_str) = rest.strip_suffix(')') {
      let digits: u32 = digits_str
        .trim()
        .parse()
        .map_err(|_| BindingError::InvalidTransform(s.to_string()))?;
      return Ok(Transform::Round(digits));
    }
  }

  if let Some(rest) = s.strip_prefix("scale(") {
    if let Some(factor_str) = rest.strip_suffix(')') {
      let factor: f64 = factor_str
        .trim()
        .parse()
        .map_err(|_| BindingError::InvalidTransform(s.to_string()))?;
      return Ok(Transform::Scale(factor));
    }
  }

  if let Some(rest) = s.strip_prefix("clamp(") {
    if let Some(args_str) = rest.strip_suffix(')') {
      let parts: Vec<&str> = args_str.split(',').collect();
      if parts.len() != 2 {
        return Err(BindingError::InvalidTransform(s.to_string()));
      }
      let min: f64 = parts[0]
        .trim()
        .parse()
        .map_err(|_| BindingError::InvalidTransform(s.to_string()))?;
      let max: f64 = parts[1]
        .trim()
        .parse()
        .map_err(|_| BindingError::InvalidTransform(s.to_string()))?;
      if min > max {
        return Err(BindingError::InvalidTransform(format!(
          "clamp({min}, {max}): min must be <= max"
        )));
      }
      return Ok(Transform::Clamp { min, max });
    }
  }

  Err(BindingError::InvalidTransform(s.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use astragauge_domain::{SensorDescriptor, SensorId, SensorSample};

  fn make_id(s: &str) -> SensorId {
    SensorId::new(s).unwrap()
  }

  async fn make_store_with_sensors(sensors: &[(&str, f64)]) -> SensorStore {
    let store = SensorStore::new();
    for (id, value) in sensors {
      let sensor_id = make_id(id);
      let descriptor = SensorDescriptor {
        id: sensor_id.clone(),
        name: format!("Sensor {}", id),
        unit: "unit".to_string(),
        category: "default".to_string(),
        device: None,
        tags: vec![],
      };
      store.register_sensor(descriptor).await.unwrap();

      let sample = SensorSample {
        sensor_id,
        value: Some(*value),
        timestamp_ms: 1000,
      };
      store.push_sample(sample).await.unwrap();
    }
    store
  }

  #[tokio::test]
  async fn test_direct_binding_returns_value() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(42.5));
    assert_eq!(result.source_count, 1);
  }

  #[tokio::test]
  async fn test_direct_binding_missing_sensor_returns_error() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("gpu.temperature"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await;
    assert!(matches!(result, Err(BindingError::UnresolvedSensor(_))));
  }

  #[tokio::test]
  async fn test_wildcard_binding_with_avg_aggregation() {
    let store = make_store_with_sensors(&[
      ("cpu.core0.temperature", 40.0),
      ("cpu.core1.temperature", 50.0),
      ("cpu.core2.temperature", 60.0),
      ("gpu.temperature", 70.0),
    ])
    .await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "cpu.core*.temperature".to_string(),
        aggregation: Aggregation::Avg,
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(50.0));
    assert_eq!(result.source_count, 3);
  }

  #[tokio::test]
  async fn test_wildcard_binding_with_max_aggregation() {
    let store = make_store_with_sensors(&[
      ("cpu.core0.temperature", 40.0),
      ("cpu.core1.temperature", 50.0),
      ("cpu.core2.temperature", 60.0),
    ])
    .await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "cpu.core*.temperature".to_string(),
        aggregation: Aggregation::Max,
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(60.0));
    assert_eq!(result.source_count, 3);
  }

  #[tokio::test]
  async fn test_wildcard_binding_no_matches_returns_error() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "gpu.*.temperature".to_string(),
        aggregation: Aggregation::Avg,
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await;
    assert!(matches!(result, Err(BindingError::WildcardNoMatch(_))));
  }

  #[tokio::test]
  async fn test_direct_binding_with_transform() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.567)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: Some("round(1)".to_string()),
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(42.6));
    assert_eq!(result.source_count, 1);
  }

  #[tokio::test]
  async fn test_wildcard_binding_with_transform() {
    let store = make_store_with_sensors(&[("cpu.core0.load", 0.5), ("cpu.core1.load", 0.7)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "cpu.core*.load".to_string(),
        aggregation: Aggregation::Avg,
      },
      transform: Some("percent".to_string()),
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(60.0));
    assert_eq!(result.source_count, 2);
  }

  #[tokio::test]
  async fn test_direct_binding_with_clamp_transform() {
    let store = make_store_with_sensors(&[("cpu.temperature", 150.0)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: Some("clamp(0, 100)".to_string()),
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(100.0));
    assert_eq!(result.source_count, 1);
  }

  #[test]
  fn test_parse_transform_abs() {
    assert_eq!(parse_transform("abs").unwrap(), Transform::Abs);
  }

  #[test]
  fn test_parse_transform_percent() {
    assert_eq!(parse_transform("percent").unwrap(), Transform::Percent);
  }

  #[test]
  fn test_parse_transform_round() {
    assert_eq!(parse_transform("round(2)").unwrap(), Transform::Round(2));
    assert_eq!(parse_transform("round(0)").unwrap(), Transform::Round(0));
    assert_eq!(parse_transform(" round(3) ").unwrap(), Transform::Round(3));
  }

  #[test]
  fn test_parse_transform_scale() {
    assert_eq!(
      parse_transform("scale(2.5)").unwrap(),
      Transform::Scale(2.5)
    );
    assert_eq!(
      parse_transform("scale(-1.0)").unwrap(),
      Transform::Scale(-1.0)
    );
  }

  #[test]
  fn test_parse_transform_clamp() {
    let result = parse_transform("clamp(0, 100)").unwrap();
    assert_eq!(
      result,
      Transform::Clamp {
        min: 0.0,
        max: 100.0
      }
    );

    let result = parse_transform("clamp(-50,50)").unwrap();
    assert_eq!(
      result,
      Transform::Clamp {
        min: -50.0,
        max: 50.0
      }
    );
  }

  #[test]
  fn test_parse_transform_invalid() {
    assert!(matches!(
      parse_transform("invalid"),
      Err(BindingError::InvalidTransform(_))
    ));
    assert!(matches!(
      parse_transform("round"),
      Err(BindingError::InvalidTransform(_))
    ));
    assert!(matches!(
      parse_transform("round(abc)"),
      Err(BindingError::InvalidTransform(_))
    ));
    assert!(matches!(
      parse_transform("clamp(100)"),
      Err(BindingError::InvalidTransform(_))
    ));
  }

  #[test]
  fn test_parse_transform_clamp_inverted_bounds() {
    let result = parse_transform("clamp(100, 0)");
    assert!(matches!(result, Err(BindingError::InvalidTransform(_))));
  }

  #[tokio::test]
  async fn test_invalid_transform_returns_error() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: Some("invalid_transform".to_string()),
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await;
    assert!(matches!(result, Err(BindingError::InvalidTransform(_))));
  }

  #[tokio::test]
  async fn test_wildcard_binding_with_count_aggregation() {
    let store = make_store_with_sensors(&[
      ("cpu.core0.temperature", 40.0),
      ("cpu.core1.temperature", 50.0),
      ("cpu.core2.temperature", 60.0),
    ])
    .await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "cpu.core*.temperature".to_string(),
        aggregation: Aggregation::Count,
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(3.0));
    assert_eq!(result.source_count, 3);
  }

  #[tokio::test]
  async fn test_wildcard_binding_with_sum_aggregation() {
    let store =
      make_store_with_sensors(&[("memory.bank0.used", 1024.0), ("memory.bank1.used", 2048.0)])
        .await;
    let engine = BindingEngine::new(store);

    let binding = Binding {
      source: BindingSource::Wildcard {
        pattern: "memory.bank*.used".to_string(),
        aggregation: Aggregation::Sum,
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await.unwrap();
    assert_eq!(result.value, Some(3072.0));
    assert_eq!(result.source_count, 2);
  }

  #[tokio::test]
  async fn test_engine_clone_and_shared_store() {
    let store = make_store_with_sensors(&[("cpu.temperature", 42.5)]).await;
    let engine1 = BindingEngine::new(store);

    let engine2 = engine1.clone();

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("cpu.temperature"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result1 = engine1.resolve(&binding).await.unwrap();
    let result2 = engine2.resolve(&binding).await.unwrap();

    assert_eq!(result1.value, result2.value);
    assert_eq!(result1.source_count, result2.source_count);
  }

  #[tokio::test]
  async fn test_from_shared_constructor() {
    let store = SensorStore::new();
    let shared = Arc::new(RwLock::new(store));

    let engine = BindingEngine::from_shared(shared.clone());

    let binding = Binding {
      source: BindingSource::Direct {
        sensor_id: make_id("nonexistent.sensor"),
      },
      transform: None,
      target_property: "value".to_string(),
    };

    let result = engine.resolve(&binding).await;
    assert!(matches!(result, Err(BindingError::UnresolvedSensor(_))));
  }
}
