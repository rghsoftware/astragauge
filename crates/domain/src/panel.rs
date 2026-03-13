use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GridConfig {
  pub columns: u8,
  pub row_height: u16,
  pub spacing: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetPlacement {
  pub id: String,
  #[serde(rename = "type")]
  pub widget_type: String,
  pub x: u8,
  pub y: u8,
  pub w: u8,
  pub h: u8,
  #[serde(default)]
  pub bindings: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelDocument {
  pub version: u8,
  pub name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub theme: Option<String>,
  pub grid: GridConfig,
  #[serde(default)]
  pub widgets: Vec<WidgetPlacement>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_panel_document_json() {
    let json = r#"{
            "version": 1,
            "name": "Mission Control",
            "theme": "default-dark",
            "grid": { "columns": 12, "row_height": 80, "spacing": 8 },
            "widgets": [
                { "id": "cpu_load", "type": "stat", "x": 0, "y": 0, "w": 3, "h": 2, "bindings": { "value": "cpu.total.utilization" } }
            ]
        }"#;

    let doc: PanelDocument = serde_json::from_str(json).unwrap();

    assert_eq!(doc.version, 1);
    assert_eq!(doc.name, "Mission Control");
    assert_eq!(doc.theme, Some("default-dark".to_string()));
    assert_eq!(doc.grid.columns, 12);
    assert_eq!(doc.grid.row_height, 80);
    assert_eq!(doc.grid.spacing, 8);
    assert_eq!(doc.widgets.len(), 1);
    assert_eq!(doc.widgets[0].id, "cpu_load");
    assert_eq!(doc.widgets[0].widget_type, "stat");
    assert_eq!(doc.widgets[0].x, 0);
    assert_eq!(doc.widgets[0].y, 0);
    assert_eq!(doc.widgets[0].w, 3);
    assert_eq!(doc.widgets[0].h, 2);
    assert_eq!(doc.widgets[0].bindings.len(), 1);
    assert_eq!(
      doc.widgets[0].bindings.get("value"),
      Some(&"cpu.total.utilization".to_string())
    );
  }

  #[test]
  fn test_panel_roundtrip() {
    let original = PanelDocument {
      version: 1,
      name: "Test Panel".to_string(),
      theme: Some("dark".to_string()),
      grid: GridConfig {
        columns: 12,
        row_height: 80,
        spacing: 8,
      },
      widgets: vec![WidgetPlacement {
        id: "widget1".to_string(),
        widget_type: "stat".to_string(),
        x: 0,
        y: 0,
        w: 2,
        h: 2,
        bindings: HashMap::from([("value".to_string(), "cpu.total.utilization".to_string())]),
      }],
    };

    let json = serde_json::to_string(&original).unwrap();
    let roundtrip: PanelDocument = serde_json::from_str(&json).unwrap();

    assert_eq!(original, roundtrip);
  }

  #[test]
  fn test_grid_config_json() {
    let json = r#"{ "columns": 12, "row_height": 80, "spacing": 8 }"#;
    let grid: GridConfig = serde_json::from_str(json).unwrap();

    assert_eq!(grid.columns, 12);
    assert_eq!(grid.row_height, 80);
    assert_eq!(grid.spacing, 8);
  }

  #[test]
  fn test_widget_placement_json() {
    let json = r#"{
            "id": "cpu_load",
            "type": "stat",
            "x": 0,
            "y": 0,
            "w": 3,
            "h": 2,
            "bindings": { "value": "cpu.total.utilization" }
        }"#;
    let widget: WidgetPlacement = serde_json::from_str(json).unwrap();

    assert_eq!(widget.id, "cpu_load");
    assert_eq!(widget.widget_type, "stat");
    assert_eq!(widget.x, 0);
    assert_eq!(widget.y, 0);
    assert_eq!(widget.w, 3);
    assert_eq!(widget.h, 2);
    assert_eq!(widget.bindings.len(), 1);
    assert_eq!(
      widget.bindings.get("value"),
      Some(&"cpu.total.utilization".to_string())
    );
  }

  #[test]
  fn test_panel_without_theme() {
    let json = r#"{
            "version": 1,
            "name": "No Theme Panel",
            "grid": { "columns": 8, "row_height": 60, "spacing": 4 },
            "widgets": []
        }"#;

    let doc: PanelDocument = serde_json::from_str(json).unwrap();
    assert_eq!(doc.theme, None);
    assert!(doc.widgets.is_empty());
  }

  #[test]
  fn test_widget_placement_empty_bindings() {
    let widget = WidgetPlacement {
      id: "widget1".to_string(),
      widget_type: "text".to_string(),
      x: 0,
      y: 0,
      w: 2,
      h: 1,
      bindings: HashMap::new(),
    };

    let json = serde_json::to_string(&widget).unwrap();
    assert!(json.contains("bindings"));
  }

  #[test]
  fn test_grid_config_roundtrip() {
    let grid = GridConfig {
      columns: 16,
      row_height: 100,
      spacing: 12,
    };

    let json = serde_json::to_string(&grid).unwrap();
    let parsed: GridConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(grid, parsed);
  }
}
