//! Widget manifest types for defining widget behavior and configuration.
//!
//! This module contains all types needed to parse and represent widget manifests
//! as defined in the widget-manifest spec.

use serde::{Deserialize, Serialize};

/// How a widget responds to container resize operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeMode {
  /// Widget has a fixed size and cannot be resized.
  Fixed,
  /// Widget scales proportionally within its container.
  Responsive,
  /// Widget maintains aspect ratio while resizing.
  AspectLocked,
}

/// Type of a widget property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
  String,
  Number,
  Boolean,
  Enum,
  ColorRole,
  FontRole,
  Size,
  Object,
  Array,
}

/// Type of value expected for a binding target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BindingValueType {
  #[serde(rename = "number")]
  Number,
  #[serde(rename = "string")]
  String,
  #[serde(rename = "boolean")]
  Boolean,
  #[serde(rename = "series<number>")]
  SeriesNumber,
  #[serde(rename = "color_role")]
  ColorRole,
  #[serde(rename = "state")]
  State,
  #[serde(rename = "timestamp")]
  Timestamp,
}

/// Kind of mock preview renderer to use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockKind {
  Stat,
  Gauge,
  Timeseries,
  List,
  Text,
  Custom,
}

/// Sizing constraints for a widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sizing {
  pub default_w: u8,
  pub default_h: u8,
  pub min_w: u8,
  pub min_h: u8,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_w: Option<u8>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_h: Option<u8>,
  pub resize_mode: ResizeMode,
}

/// A configurable property for a widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property {
  pub key: String,
  pub label: String,
  #[serde(rename = "type")]
  pub property_type: PropertyType,
  pub required: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub default: Option<serde_json::Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub options: Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub group: Option<String>,
}

/// A binding target for connecting sensors to widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingTarget {
  pub key: String,
  pub label: String,
  pub value_type: BindingValueType,
  pub required: bool,
  pub multi: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

/// Preview configuration for the widget editor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preview {
  pub mock_kind: MockKind,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub sample_props: Option<serde_json::Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub sample_bindings: Option<serde_json::Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub placeholder_label: Option<String>,
}

/// Theming capabilities for a widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theming {
  pub supports_background: bool,
  pub supports_accent: bool,
  pub supports_threshold_colors: bool,
  pub supports_typography_roles: bool,
  #[serde(default)]
  pub style_slots: Vec<String>,
}

/// Feature capabilities for a widget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capabilities {
  pub supports_history: bool,
  pub supports_thresholds: bool,
  pub supports_multiple_series: bool,
  pub supports_secondary_text: bool,
  pub supports_overlay: bool,
}

/// Validation rules for widget configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Validation {
  pub requires_value_binding: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub min_supported_bindings: Option<u8>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_supported_bindings: Option<u8>,
  #[serde(default)]
  pub required_props: Vec<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub layout_rules: Option<serde_json::Value>,
}

/// Complete widget manifest defining behavior and configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetManifest {
  pub id: String,
  pub name: String,
  pub category: String,
  pub version: u8,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub sizing: Sizing,
  #[serde(default)]
  pub properties: Vec<Property>,
  #[serde(default)]
  pub bindings: Vec<BindingTarget>,
  pub preview: Preview,
  pub theming: Theming,
  pub capabilities: Capabilities,
  pub validation: Validation,
}

#[cfg(test)]
mod tests {
  use super::*;

  const STAT_TILE_MANIFEST: &str = r#"{
        "id": "core.stat",
        "name": "Stat Tile",
        "category": "basic",
        "version": 1,
        "description": "Displays a primary value with optional label and unit.",
        "sizing": {
            "default_w": 3,
            "default_h": 2,
            "min_w": 2,
            "min_h": 1,
            "resize_mode": "responsive"
        },
        "properties": [
            { "key": "label", "label": "Label", "type": "string", "required": false, "default": "" }
        ],
        "bindings": [
            { "key": "value", "label": "Value", "value_type": "number", "required": true, "multi": false }
        ],
        "preview": { "mock_kind": "stat", "sample_props": { "label": "CPU" }, "sample_bindings": { "value": 43.2 } },
        "theming": { "supports_background": true, "supports_accent": true, "supports_threshold_colors": true, "supports_typography_roles": true, "style_slots": ["surface", "value_text", "label_text"] },
        "capabilities": { "supports_history": false, "supports_thresholds": true, "supports_multiple_series": false, "supports_secondary_text": true, "supports_overlay": false },
        "validation": { "requires_value_binding": true, "min_supported_bindings": 1, "max_supported_bindings": 2 }
    }"#;

  const SPARKLINE_MANIFEST: &str = r#"{
        "id": "core.sparkline",
        "name": "Sparkline",
        "category": "charts",
        "version": 1,
        "description": "Displays a compact historical trend line.",
        "sizing": { "default_w": 4, "default_h": 2, "min_w": 3, "min_h": 1, "resize_mode": "responsive" },
        "properties": [{ "key": "show_fill", "label": "Show Fill", "type": "boolean", "required": false, "default": false }],
        "bindings": [{ "key": "series", "label": "Series", "value_type": "series<number>", "required": true, "multi": false }],
        "preview": { "mock_kind": "timeseries", "sample_bindings": { "series": [12, 18, 23, 19, 25, 28, 21] } },
        "theming": { "supports_background": true, "supports_accent": true, "supports_threshold_colors": false, "supports_typography_roles": false, "style_slots": ["surface", "trend_line", "trend_fill"] },
        "capabilities": { "supports_history": true, "supports_thresholds": false, "supports_multiple_series": false, "supports_secondary_text": false, "supports_overlay": false },
        "validation": { "requires_value_binding": false, "min_supported_bindings": 1, "max_supported_bindings": 1 }
    }"#;

  #[test]
  fn parse_stat_tile_manifest() {
    let manifest: WidgetManifest =
      serde_json::from_str(STAT_TILE_MANIFEST).expect("Failed to parse stat tile manifest");

    assert_eq!(manifest.id, "core.stat");
    assert_eq!(manifest.name, "Stat Tile");
    assert_eq!(manifest.category, "basic");
    assert_eq!(manifest.version, 1);
    assert_eq!(
      manifest.description,
      Some("Displays a primary value with optional label and unit.".to_string())
    );

    assert_eq!(manifest.sizing.default_w, 3);
    assert_eq!(manifest.sizing.default_h, 2);
    assert_eq!(manifest.sizing.min_w, 2);
    assert_eq!(manifest.sizing.min_h, 1);
    assert_eq!(manifest.sizing.max_w, None);
    assert_eq!(manifest.sizing.max_h, None);
    assert_eq!(manifest.sizing.resize_mode, ResizeMode::Responsive);

    assert_eq!(manifest.properties.len(), 1);
    assert_eq!(manifest.properties[0].key, "label");
    assert_eq!(manifest.properties[0].property_type, PropertyType::String);

    assert_eq!(manifest.bindings.len(), 1);
    assert_eq!(manifest.bindings[0].key, "value");
    assert_eq!(manifest.bindings[0].value_type, BindingValueType::Number);
    assert!(manifest.bindings[0].required);
    assert!(!manifest.bindings[0].multi);

    assert_eq!(manifest.preview.mock_kind, MockKind::Stat);
    assert!(manifest.preview.sample_props.is_some());

    assert!(manifest.theming.supports_background);
    assert!(manifest.theming.supports_accent);
    assert_eq!(
      manifest.theming.style_slots,
      vec!["surface", "value_text", "label_text"]
    );

    assert!(!manifest.capabilities.supports_history);
    assert!(manifest.capabilities.supports_thresholds);

    assert!(manifest.validation.requires_value_binding);
    assert_eq!(manifest.validation.min_supported_bindings, Some(1));
    assert_eq!(manifest.validation.max_supported_bindings, Some(2));
  }

  #[test]
  fn parse_sparkline_manifest() {
    let manifest: WidgetManifest =
      serde_json::from_str(SPARKLINE_MANIFEST).expect("Failed to parse sparkline manifest");

    assert_eq!(manifest.id, "core.sparkline");
    assert_eq!(manifest.name, "Sparkline");
    assert_eq!(manifest.category, "charts");
    assert_eq!(manifest.version, 1);

    assert_eq!(manifest.sizing.default_w, 4);
    assert_eq!(manifest.sizing.resize_mode, ResizeMode::Responsive);

    assert_eq!(manifest.properties.len(), 1);
    assert_eq!(manifest.properties[0].key, "show_fill");
    assert_eq!(manifest.properties[0].property_type, PropertyType::Boolean);

    assert_eq!(manifest.bindings.len(), 1);
    assert_eq!(manifest.bindings[0].key, "series");
    assert_eq!(
      manifest.bindings[0].value_type,
      BindingValueType::SeriesNumber
    );

    assert_eq!(manifest.preview.mock_kind, MockKind::Timeseries);
    assert!(manifest.preview.sample_props.is_none());

    assert!(manifest.theming.supports_background);
    assert!(!manifest.theming.supports_threshold_colors);
    assert!(!manifest.theming.supports_typography_roles);

    assert!(manifest.capabilities.supports_history);
    assert!(!manifest.capabilities.supports_thresholds);
  }

  #[test]
  fn serialize_deserialize_roundtrip() {
    let original: WidgetManifest = serde_json::from_str(STAT_TILE_MANIFEST).unwrap();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: WidgetManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(original, parsed);
  }

  #[test]
  fn resize_mode_variants() {
    let fixed = r#""fixed""#;
    let responsive = r#""responsive""#;
    let aspect_locked = r#""aspect_locked""#;

    assert_eq!(
      serde_json::from_str::<ResizeMode>(fixed).unwrap(),
      ResizeMode::Fixed
    );
    assert_eq!(
      serde_json::from_str::<ResizeMode>(responsive).unwrap(),
      ResizeMode::Responsive
    );
    assert_eq!(
      serde_json::from_str::<ResizeMode>(aspect_locked).unwrap(),
      ResizeMode::AspectLocked
    );
  }

  #[test]
  fn property_type_variants() {
    assert_eq!(
      serde_json::from_str::<PropertyType>(r#""string""#).unwrap(),
      PropertyType::String
    );
    assert_eq!(
      serde_json::from_str::<PropertyType>(r#""color_role""#).unwrap(),
      PropertyType::ColorRole
    );
  }

  #[test]
  fn mock_kind_variants() {
    assert_eq!(
      serde_json::from_str::<MockKind>(r#""stat""#).unwrap(),
      MockKind::Stat
    );
    assert_eq!(
      serde_json::from_str::<MockKind>(r#""timeseries""#).unwrap(),
      MockKind::Timeseries
    );
    assert_eq!(
      serde_json::from_str::<MockKind>(r#""custom""#).unwrap(),
      MockKind::Custom
    );
  }

  #[test]
  fn binding_value_type_variants() {
    assert_eq!(
      serde_json::from_str::<BindingValueType>(r#""number""#).unwrap(),
      BindingValueType::Number
    );
    assert_eq!(
      serde_json::from_str::<BindingValueType>(r#""series<number>""#).unwrap(),
      BindingValueType::SeriesNumber
    );
  }

  #[test]
  fn sizing_with_max_constraints() {
    let json = r#"{
            "default_w": 2,
            "default_h": 2,
            "min_w": 1,
            "min_h": 1,
            "max_w": 4,
            "max_h": 4,
            "resize_mode": "fixed"
        }"#;

    let sizing: Sizing = serde_json::from_str(json).unwrap();
    assert_eq!(sizing.max_w, Some(4));
    assert_eq!(sizing.max_h, Some(4));
    assert_eq!(sizing.resize_mode, ResizeMode::Fixed);
  }

  #[test]
  fn minimal_manifest() {
    let json = r#"{
            "id": "test.widget",
            "name": "Test Widget",
            "category": "test",
            "version": 1,
            "sizing": {
                "default_w": 1,
                "default_h": 1,
                "min_w": 1,
                "min_h": 1,
                "resize_mode": "fixed"
            },
            "preview": { "mock_kind": "text" },
            "theming": {
                "supports_background": false,
                "supports_accent": false,
                "supports_threshold_colors": false,
                "supports_typography_roles": false
            },
            "capabilities": {
                "supports_history": false,
                "supports_thresholds": false,
                "supports_multiple_series": false,
                "supports_secondary_text": false,
                "supports_overlay": false
            },
            "validation": { "requires_value_binding": false }
        }"#;

    let manifest: WidgetManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.id, "test.widget");
    assert_eq!(manifest.description, None);
    assert!(manifest.properties.is_empty());
    assert!(manifest.bindings.is_empty());
    assert!(manifest.validation.required_props.is_empty());
    assert!(manifest.validation.layout_rules.is_none());
  }
}
