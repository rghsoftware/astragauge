use crate::types::Aggregation;

impl Aggregation {
  /// Apply the aggregation function to a slice of optional values.
  ///
  /// Returns None if the input is empty or contains only None values.
  pub fn apply(&self, values: &[Option<f64>]) -> Option<f64> {
    let valid: Vec<f64> = values.iter().filter_map(|&v| v).collect();

    if valid.is_empty() {
      return None;
    }

    Some(match self {
      Aggregation::Avg => valid.iter().sum::<f64>() / valid.len() as f64,
      Aggregation::Min => valid.iter().cloned().fold(f64::INFINITY, f64::min),
      Aggregation::Max => valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
      Aggregation::Sum => valid.iter().sum(),
      Aggregation::Count => valid.len() as f64,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn avg_basic() {
    let values = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result, Some(3.0));
  }

  #[test]
  fn min_basic() {
    let values = vec![Some(5.0), Some(2.0), Some(8.0), Some(1.0), Some(3.0)];
    let result = Aggregation::Min.apply(&values);
    assert_eq!(result, Some(1.0));
  }

  #[test]
  fn max_basic() {
    let values = vec![Some(5.0), Some(2.0), Some(8.0), Some(1.0), Some(3.0)];
    let result = Aggregation::Max.apply(&values);
    assert_eq!(result, Some(8.0));
  }

  #[test]
  fn sum_basic() {
    let values = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result, Some(15.0));
  }

  #[test]
  fn count_basic() {
    let values = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)];
    let result = Aggregation::Count.apply(&values);
    assert_eq!(result, Some(5.0));
  }

  #[test]
  fn empty_slice_returns_none() {
    let values: Vec<Option<f64>> = vec![];
    assert_eq!(Aggregation::Avg.apply(&values), None);
    assert_eq!(Aggregation::Min.apply(&values), None);
    assert_eq!(Aggregation::Max.apply(&values), None);
    assert_eq!(Aggregation::Sum.apply(&values), None);
    assert_eq!(Aggregation::Count.apply(&values), None);
  }

  #[test]
  fn all_none_returns_none() {
    let values = vec![None, None, None, None];
    assert_eq!(Aggregation::Avg.apply(&values), None);
    assert_eq!(Aggregation::Min.apply(&values), None);
    assert_eq!(Aggregation::Max.apply(&values), None);
    assert_eq!(Aggregation::Sum.apply(&values), None);
    assert_eq!(Aggregation::Count.apply(&values), None);
  }

  #[test]
  fn single_value_all_aggregations() {
    let values = vec![Some(42.0)];
    assert_eq!(Aggregation::Avg.apply(&values), Some(42.0));
    assert_eq!(Aggregation::Min.apply(&values), Some(42.0));
    assert_eq!(Aggregation::Max.apply(&values), Some(42.0));
    assert_eq!(Aggregation::Sum.apply(&values), Some(42.0));
    assert_eq!(Aggregation::Count.apply(&values), Some(1.0));
  }

  #[test]
  fn avg_with_none_values() {
    let values = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result, Some(3.0));
  }

  #[test]
  fn avg_mixed_positive_negative() {
    let values = vec![Some(-10.0), Some(0.0), Some(10.0), Some(20.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result, Some(5.0));
  }

  #[test]
  fn min_with_negative_values() {
    let values = vec![Some(-5.0), Some(-10.0), Some(0.0), Some(5.0)];
    let result = Aggregation::Min.apply(&values);
    assert_eq!(result, Some(-10.0));
  }

  #[test]
  fn max_with_negative_values() {
    let values = vec![Some(-5.0), Some(-10.0), Some(0.0), Some(5.0)];
    let result = Aggregation::Max.apply(&values);
    assert_eq!(result, Some(5.0));
  }

  #[test]
  fn sum_with_negative_values() {
    let values = vec![Some(10.0), Some(-5.0), Some(-3.0), Some(2.0)];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result, Some(4.0));
  }

  #[test]
  fn sum_with_none_values() {
    let values = vec![Some(10.0), None, Some(5.0), None];
    let result = Aggregation::Sum.apply(&values);
    assert_eq!(result, Some(15.0));
  }

  #[test]
  fn count_only_non_none() {
    let values = vec![Some(1.0), None, Some(2.0), None, Some(3.0), None];
    let result = Aggregation::Count.apply(&values);
    assert_eq!(result, Some(3.0));
  }

  #[test]
  fn count_zero_values_returns_one() {
    let values = vec![Some(0.0)];
    let result = Aggregation::Count.apply(&values);
    assert_eq!(result, Some(1.0));
  }

  #[test]
  fn avg_with_floating_point() {
    let values = vec![Some(1.0), Some(2.0), Some(2.0)];
    let result = Aggregation::Avg.apply(&values);
    assert_eq!(result, Some(1.6666666666666667));
  }

  #[test]
  fn min_max_identical_values() {
    let values = vec![Some(5.0), Some(5.0), Some(5.0)];
    assert_eq!(Aggregation::Min.apply(&values), Some(5.0));
    assert_eq!(Aggregation::Max.apply(&values), Some(5.0));
  }
}
