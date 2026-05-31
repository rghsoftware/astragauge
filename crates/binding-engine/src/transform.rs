use crate::types::Transform;

impl Transform {
  /// Apply a transform to an optional sensor value.
  ///
  /// Returns `None` if input is `None` (null propagation).
  /// Returns `Some(transformed_value)` if input is `Some(value)`.
  ///
  /// # Null Propagation
  /// - None input → None output
  /// - This prevents errors from propagating when sensors are unavailable
  pub fn apply(&self, value: Option<f64>) -> Option<f64> {
    let v = value?;
    Some(match self {
      Transform::Round(digits) => {
        let factor = 10f64.powi(*digits as i32);
        (v * factor).round() / factor
      }
      Transform::Clamp { min, max } => v.clamp(*min, *max),
      Transform::Abs => v.abs(),
      Transform::Scale(factor) => v * factor,
      Transform::Percent => v * 100.0,
      Transform::CelsiusToFahrenheit => v * 9.0 / 5.0 + 32.0,
      Transform::BytesToKb => v / 1024.0,
      Transform::BytesToMb => v / (1024.0 * 1024.0),
      Transform::BytesToGb => v / (1024.0 * 1024.0 * 1024.0),
      Transform::BytesToTb => v / (1024.0 * 1024.0 * 1024.0 * 1024.0),
      Transform::BitsToKbit => v / 1000.0,
      Transform::BitsToMbit => v / 1_000_000.0,
      Transform::BitsToGbit => v / 1_000_000_000.0,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ===== NULL PROPAGATION =====

  #[test]
  fn test_none_input_produces_none_output() {
    let transforms = [
      Transform::Round(2),
      Transform::Clamp {
        min: 0.0,
        max: 100.0,
      },
      Transform::Abs,
      Transform::Scale(2.0),
      Transform::Percent,
      Transform::CelsiusToFahrenheit,
      Transform::BytesToMb,
      Transform::BitsToMbit,
    ];

    for transform in transforms {
      assert_eq!(transform.apply(None), None);
    }
  }

  // ===== ROUND TRANSFORM =====

  #[test]
  fn test_round_zero_digits() {
    let transform = Transform::Round(0);
    assert_eq!(transform.apply(Some(3.567)), Some(4.0));
    assert_eq!(transform.apply(Some(3.499)), Some(3.0));
    assert_eq!(transform.apply(Some(-3.5)), Some(-4.0));
  }

  #[test]
  fn test_round_one_digit() {
    let transform = Transform::Round(1);
    assert_eq!(transform.apply(Some(3.16159)), Some(3.2));
    assert_eq!(transform.apply(Some(3.19)), Some(3.2));
    assert_eq!(transform.apply(Some(-3.12)), Some(-3.1));
  }

  #[test]
  fn test_round_two_digits() {
    let transform = Transform::Round(2);
    assert_eq!(transform.apply(Some(3.16159)), Some(3.16));
    assert_eq!(transform.apply(Some(3.145)), Some(3.15));
    assert_eq!(transform.apply(Some(0.9999)), Some(1.0));
  }

  #[test]
  fn test_round_ten_digits() {
    let transform = Transform::Round(10);
    assert_eq!(transform.apply(Some(1.23456789012345)), Some(1.2345678901));
  }

  #[test]
  fn test_round_negative_value() {
    let transform = Transform::Round(2);
    assert_eq!(transform.apply(Some(-3.12159)), Some(-3.12));
  }

  #[test]
  fn test_round_zero() {
    let transform = Transform::Round(2);
    assert_eq!(transform.apply(Some(0.0)), Some(0.0));
  }

  // ===== CLAMP TRANSFORM =====

  #[test]
  fn test_clamp_within_range() {
    let transform = Transform::Clamp {
      min: 0.0,
      max: 100.0,
    };
    assert_eq!(transform.apply(Some(50.0)), Some(50.0));
    assert_eq!(transform.apply(Some(25.5)), Some(25.5));
  }

  #[test]
  fn test_clamp_at_min_bound() {
    let transform = Transform::Clamp {
      min: 0.0,
      max: 100.0,
    };
    assert_eq!(transform.apply(Some(0.0)), Some(0.0));
    assert_eq!(transform.apply(Some(-10.0)), Some(0.0));
  }

  #[test]
  fn test_clamp_at_max_bound() {
    let transform = Transform::Clamp {
      min: 0.0,
      max: 100.0,
    };
    assert_eq!(transform.apply(Some(100.0)), Some(100.0));
    assert_eq!(transform.apply(Some(150.0)), Some(100.0));
  }

  #[test]
  fn test_clamp_negative_range() {
    let transform = Transform::Clamp {
      min: -50.0,
      max: -10.0,
    };
    assert_eq!(transform.apply(Some(-30.0)), Some(-30.0));
    assert_eq!(transform.apply(Some(-60.0)), Some(-50.0));
    assert_eq!(transform.apply(Some(0.0)), Some(-10.0));
  }

  #[test]
  fn test_clamp_valid_bounds() {
    let transform = Transform::Clamp {
      min: 0.0,
      max: 100.0,
    };
    assert_eq!(transform.apply(Some(50.0)), Some(50.0));
  }

  // ===== ABS TRANSFORM =====

  #[test]
  fn test_abs_positive_value() {
    let transform = Transform::Abs;
    assert_eq!(transform.apply(Some(42.5)), Some(42.5));
  }

  #[test]
  fn test_abs_negative_value() {
    let transform = Transform::Abs;
    assert_eq!(transform.apply(Some(-42.5)), Some(42.5));
  }

  #[test]
  fn test_abs_zero() {
    let transform = Transform::Abs;
    assert_eq!(transform.apply(Some(0.0)), Some(0.0));
  }

  // ===== SCALE TRANSFORM =====

  #[test]
  fn test_scale_positive_factor() {
    let transform = Transform::Scale(2.5);
    assert_eq!(transform.apply(Some(10.0)), Some(25.0));
    assert_eq!(transform.apply(Some(-4.0)), Some(-10.0));
  }

  #[test]
  fn test_scale_negative_factor() {
    let transform = Transform::Scale(-2.0);
    assert_eq!(transform.apply(Some(10.0)), Some(-20.0));
    assert_eq!(transform.apply(Some(-5.0)), Some(10.0));
  }

  #[test]
  fn test_scale_zero_factor() {
    let transform = Transform::Scale(0.0);
    assert_eq!(transform.apply(Some(42.0)), Some(0.0));
    assert_eq!(transform.apply(Some(-42.0)), Some(0.0));
  }

  #[test]
  fn test_scale_fractional_factor() {
    let transform = Transform::Scale(0.5);
    assert_eq!(transform.apply(Some(10.0)), Some(5.0));
    assert_eq!(transform.apply(Some(-8.0)), Some(-4.0));
  }

  // ===== PERCENT TRANSFORM =====

  #[test]
  fn test_percent_from_fraction() {
    let transform = Transform::Percent;
    assert_eq!(transform.apply(Some(0.5)), Some(50.0));
    assert_eq!(transform.apply(Some(0.75)), Some(75.0));
  }

  #[test]
  fn test_percent_from_whole_number() {
    let transform = Transform::Percent;
    assert_eq!(transform.apply(Some(1.0)), Some(100.0));
    assert_eq!(transform.apply(Some(2.5)), Some(250.0));
  }

  #[test]
  fn test_infinity_propagates_for_non_clamp() {
    let transforms = [
      Transform::Round(2),
      Transform::Abs,
      Transform::Scale(2.0),
      Transform::Percent,
    ];

    for transform in transforms {
      let result = transform.apply(Some(f64::INFINITY));
      assert!(result.map(|v| v.is_infinite()).unwrap_or(false));
    }
  }

  #[test]
  fn test_neg_infinity_propagates_for_non_clamp() {
    let transforms = [
      Transform::Round(2),
      Transform::Abs,
      Transform::Scale(2.0),
      Transform::Percent,
    ];

    for transform in transforms {
      let result = transform.apply(Some(f64::NEG_INFINITY));
      assert!(result.map(|v| v.is_infinite()).unwrap_or(false));
    }
  }

  #[test]
  fn test_percent_zero() {
    let transform = Transform::Percent;
    assert_eq!(transform.apply(Some(0.0)), Some(0.0));
  }

  // ===== SPECIAL VALUES =====

  #[test]
  fn test_nan_propagates() {
    let transforms = [
      Transform::Round(2),
      Transform::Clamp {
        min: 0.0,
        max: 100.0,
      },
      Transform::Abs,
      Transform::Scale(2.0),
      Transform::Percent,
    ];

    for transform in transforms {
      let result = transform.apply(Some(f64::NAN));
      assert!(result.map(|v| v.is_nan()).unwrap_or(false));
    }
  }

  #[test]
  fn test_clamp_infinity() {
    let transform = Transform::Clamp {
      min: 0.0,
      max: 100.0,
    };
    assert_eq!(transform.apply(Some(f64::INFINITY)), Some(100.0));
    assert_eq!(transform.apply(Some(f64::NEG_INFINITY)), Some(0.0));
  }

  // ===== COMBINED EDGE CASES =====

  #[test]
  fn test_scale_then_percent_equivalence() {
    let value = Some(0.5);
    let scaled = Transform::Scale(100.0).apply(value);
    let percent = Transform::Percent.apply(value);
    assert_eq!(scaled, percent);
  }

  #[test]
  fn test_round_zero_scale() {
    let rounded = Transform::Round(0).apply(Some(0.5));
    assert_eq!(rounded, Some(1.0));
  }

  // ===== UNIT CONVERSION TRANSFORMS =====

  #[test]
  fn test_celsius_to_fahrenheit_freezing() {
    assert_eq!(Transform::CelsiusToFahrenheit.apply(Some(0.0)), Some(32.0));
  }

  #[test]
  fn test_celsius_to_fahrenheit_boiling() {
    assert_eq!(
      Transform::CelsiusToFahrenheit.apply(Some(100.0)),
      Some(212.0)
    );
  }

  #[test]
  fn test_celsius_to_fahrenheit_negative() {
    assert_eq!(
      Transform::CelsiusToFahrenheit.apply(Some(-40.0)),
      Some(-40.0)
    );
  }

  #[test]
  fn test_celsius_to_fahrenheit_body_temp() {
    assert_eq!(
      Transform::CelsiusToFahrenheit.apply(Some(37.0)),
      Some(98.6)
    );
  }

  #[test]
  fn test_bytes_to_kb() {
    assert_eq!(Transform::BytesToKb.apply(Some(1024.0)), Some(1.0));
    assert_eq!(Transform::BytesToKb.apply(Some(2048.0)), Some(2.0));
  }

  #[test]
  fn test_bytes_to_mb() {
    assert_eq!(
      Transform::BytesToMb.apply(Some(1024.0 * 1024.0)),
      Some(1.0)
    );
    assert_eq!(Transform::BytesToMb.apply(Some(5.0 * 1024.0 * 1024.0)), Some(5.0));
  }

  #[test]
  fn test_bytes_to_gb() {
    assert_eq!(
      Transform::BytesToGb.apply(Some(1024.0 * 1024.0 * 1024.0)),
      Some(1.0)
    );
  }

  #[test]
  fn test_bytes_to_tb() {
    assert_eq!(
      Transform::BytesToTb.apply(Some(1024.0 * 1024.0 * 1024.0 * 1024.0)),
      Some(1.0)
    );
  }

  #[test]
  fn test_bits_to_kbit() {
    assert_eq!(Transform::BitsToKbit.apply(Some(1000.0)), Some(1.0));
  }

  #[test]
  fn test_bits_to_mbit() {
    assert_eq!(Transform::BitsToMbit.apply(Some(1_000_000.0)), Some(1.0));
  }

  #[test]
  fn test_bits_to_gbit() {
    assert_eq!(Transform::BitsToGbit.apply(Some(1_000_000_000.0)), Some(1.0));
  }

  #[test]
  fn test_bytes_zero() {
    assert_eq!(Transform::BytesToMb.apply(Some(0.0)), Some(0.0));
    assert_eq!(Transform::BytesToGb.apply(Some(0.0)), Some(0.0));
  }

  #[test]
  fn test_unit_conversions_propagate_nan() {
    assert!(Transform::CelsiusToFahrenheit.apply(Some(f64::NAN)).unwrap().is_nan());
    assert!(Transform::BytesToMb.apply(Some(f64::NAN)).unwrap().is_nan());
  }

  #[test]
  fn test_unit_conversions_propagate_infinity() {
    assert!(Transform::CelsiusToFahrenheit.apply(Some(f64::INFINITY)).unwrap().is_infinite());
    assert!(Transform::BytesToGb.apply(Some(f64::INFINITY)).unwrap().is_infinite());
  }
}
