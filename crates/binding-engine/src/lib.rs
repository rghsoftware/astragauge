//! AstraGauge Binding Engine
//!
//! Resolves sensor values to widget inputs with transforms and aggregations.

pub mod aggregation;
pub mod engine;
pub mod subscription;
pub mod transform;
pub mod types;

pub use crate::aggregation::AggregationResult;
pub use crate::engine::{parse_transform, BindingEngine};
pub use crate::subscription::BindingSubscription;
pub use crate::types::{
  Aggregation, Binding, BindingError, BindingResult, BindingSource, ResolvedBinding, Transform,
};
