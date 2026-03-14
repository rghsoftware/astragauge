use crate::types::Aggregation;

/// Result of applying an aggregation function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AggregationResult {
  /// The aggregated value (None if no valid values)
  pub value: Option<f64>,
  /// Number of valid (non-None, non-NaN) values that contributed
  pub valid_count: usize,
}

impl Aggregation {
  /// Apply the aggregation function to a slice of optional values.
  ///
  /// Filters out None and NaN values. Returns an `AggregationResult` containing
  /// the aggregated value and the count of valid values that contributed.
  ///
  /// # NaN Handling
  ///
  /// NaN values are treated as missing data and filtered out, consistent with
  /// how None values are handled. This prevents NaN propagation through
  /// aggregations like `Avg` where a single NaN would corrupt the result.
  pub fn apply(&self, values: &[Option<f64>]) -> AggregationResult {
    // Filter out None and NaN values
    let valid: Vec<f64> = values
      .iter()
      .filter_map(|&v| v.filter(|x| !x.is_nan()))
      .collect();

    let valid_count = valid.len();

    if valid.is_empty() {
      return AggregationResult {
        value: None,
        valid_count: 0,
      };
    }

    let value = Some(match self {
      Aggregation::Avg => valid.iter().sum::<f64>() / valid.len() as f64,
      Aggregation::Min => valid.iter().cloned().fold(f64::INFINITY, f64::min),
      Aggregation::Max => valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
      Aggregation::Sum => valid.iter().sum(),
      Aggregation::Count => valid.len() as f64,
    });

    AggregationResult { value, valid_count }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn apply_agg(agg: &Aggregation, values: &[Option<f64>]) -> Option<f64> {
    agg.apply(values).value
  }

  #[test]
  fn avg_basic() {
    let values = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, Some(3.0));
    assert_eq!(result.valid_count, 5);
  }

  #[test]
  fn min_basic() {
    let values = vec![Some(5.0), Some(2.0), Some(8.0), Some(1.0), Some(3.0)];
    let result = Aggregation::Min.apply(&values);
    assert_eq!(result.value, Some(1.0));
    assert_eq!(result.valid_count, 5);
  }

  #[test]
  fn max_basic() {
    let values = vec![Some(5.0), Some(2.0), Some(8.0), Some(1.0), Some(3.0)];
    let result = Aggregation::Max.apply(&values);
    assert_eq!(result.value, Some(8.0));
    assert_eq!(result.valid_count, 5);
  }

  #[test]
  fn sum_basic() {
    let values = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result.value, Some(15.0));
    assert_eq!(result.valid_count, 5);
  }

  #[test]
  fn count_basic() {
    let values = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let result = Aggregation::Count.apply(&values);
    assert_eq!(result.value, Some(5.0));
    assert_eq!(result.valid_count, 5);
  }

  #[test]
  fn empty_slice_returns_none() {
    let values: Vec<Option<f64>> = vec![];
    assert_eq!(apply_agg(&Aggregation::Avg, &values), None);
    assert_eq!(apply_agg(&Aggregation::Min, &values), None);
    assert_eq!(apply_agg(&Aggregation::Max, &values), None);
    assert_eq!(apply_agg(&Aggregation::Sum, &values), None);
    assert_eq!(apply_agg(&Aggregation::Count, &values), None);
  }

  #[test]
  fn all_none_returns_none() {
    let values = vec![None, None, None, None];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, None);
    assert_eq!(result.valid_count, 0);
  }

  #[test]
  fn single_value_all_aggregations() {
    let values = vec![Some(42.0)];
    assert_eq!(apply_agg(&Aggregation::Avg, &values), Some(42.0));
    assert_eq!(apply_agg(&Aggregation::Min, &values), Some(42.0));
    assert_eq!(apply_agg(&Aggregation::Max, &values), Some(42.0));
    assert_eq!(apply_agg(&Aggregation::Sum, &values), Some(42.0));
    assert_eq!(apply_agg(&Aggregation::Count, &values), Some(1.0));
  }

  #[test]
  fn avg_with_none_values() {
    let values = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, Some(3.0));
    assert_eq!(result.valid_count, 3);
  }

  #[test]
  fn avg_mixed_positive_negative() {
    let values = vec![Some(-10.0), Some(0.0), Some(10.0), Some(20.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, Some(5.0));
    assert_eq!(result.valid_count, 4);
  }

  #[test]
  fn min_with_negative_values() {
    let values = vec![Some(-5.0), Some(-10.0), Some(0.0), Some(5.0)];
    let result = Aggregation::Min.apply(&values);
    assert_eq!(result.value, Some(-10.0));
    assert_eq!(result.valid_count, 4);
  }

  #[test]
  fn max_with_negative_values() {
    let values = vec![Some(-5.0), Some(-10.0), Some(0.0), Some(5.0)];
    let result = Aggregation::Max.apply(&values);
    assert_eq!(result.value, Some(5.0));
    assert_eq!(result.valid_count, 4);
  }

  #[test]
  fn sum_with_negative_values() {
    let values = vec![Some(10.0), Some(-5.0), Some(-3.0), Some(2.0)];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result.value, Some(4.0));
    assert_eq!(result.valid_count, 4);
  }

  #[test]
  fn sum_with_none_values() {
    let values = vec![Some(10.0), None, Some(5.0), None];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result.value, Some(15.0));
    assert_eq!(result.valid_count, 2);
  }

  #[test]
  fn count_only_non_none() {
    let values = vec![Some(1.0), None, Some(2.0), None, Some(3.0), None];
    let result = Aggregation::Count.apply(&values);
    assert_eq!(result.value, Some(3.0));
    assert_eq!(result.valid_count, 3);
  }

  #[test]
  fn count_zero_values_returns_one() {
    let values = vec![Some(0.0)];
    let result = Aggregation::Count.apply(&values);
    assert_eq!(result.value, Some(1.0));
    assert_eq!(result.valid_count, 1);
  }

  #[test]
  fn avg_with_floating_point() {
    let values = vec![Some(1.0), Some(2.0), Some(2.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, Some(1.6666666666666667));
    assert_eq!(result.valid_count, 3);
  }

  #[test]
  fn min_max_identical_values() {
    let values = vec![Some(5.0), Some(5.0), Some(5.0)];
    assert_eq!(apply_agg(&Aggregation::Min, &values), Some(5.0));
    assert_eq!(apply_agg(&Aggregation::Max, &values), Some(5.0));
  }

  #[test]
  fn nan_values_filtered_out() {
    let values = vec![
      Some(1.0),
      Some(f64::NAN),
      Some(3.0),
      Some(f64::NAN),
      Some(5.0),
    ];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, Some(3.0));
    assert_eq!(result.valid_count, 3);
  }

  #[test]
  fn all_nan_returns_none() {
    let values = vec![Some(f64::NAN), Some(f64::NAN), Some(f64::NAN)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result.value, None);
    assert_eq!(result.valid_count, 0);
  }

  #[test]
  fn mixed_none_and_nan_filtered() {
    let values = vec![Some(10.0), None, Some(f64::NAN), Some(20.0), None];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result.value, Some(30.0));
    assert_eq!(result.valid_count, 2);
  }

  #[test]
  fn min_with_nan_ignores_nan() {
    let values = vec![Some(5.0), Some(f64::NAN), Some(2.0)];
    let result = Aggregation::Min.apply(&values);
    assert_eq!(result.value, Some(2.0));
    assert_eq!(result.valid_count, 2);
  }

  #[test]
  fn max_with_nan_ignores_nan() {
    let values = vec![Some(5.0), Some(f64::NAN), Some(10.0)];
    let result = Aggregation::Max.apply(&values);
    assert_eq!(result.value, Some(10.0));
    assert_eq!(result.valid_count, 2);
  }
}
