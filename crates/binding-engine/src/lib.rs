//! AstraGauge Binding Engine
//!
//! Resolves sensor values to widget inputs with transforms and aggregations.

pub mod aggregation;
pub mod engine;
pub mod subscription;
pub mod transform;
pub mod types;

pub use crate::aggregation::AggregationResult;
pub use crate::engine::{apply_transforms, parse_transform, parse_transforms, BindingEngine};
pub use crate::subscription::BindingSubscription;
pub use crate::types::{
  Aggregation, Binding, BindingError, BindingResult, BindingSource, FormattedBinding, FormatSpec,
  ResolvedBinding, Transform, format_value,
};
