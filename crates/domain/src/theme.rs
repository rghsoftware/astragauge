use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ColorRoles {
  pub background: String,
  pub surface: String,
  pub text_primary: String,
  pub text_secondary: String,
  pub accent: String,
  pub good: String,
  pub warn: String,
  pub critical: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Typography {
  pub primary_font: String,
  pub numeric_font: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Spacing {
  pub grid_spacing: u8,
  pub widget_padding: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThemeDocument {
  pub name: String,
  pub version: u8,
  pub colors: ColorRoles,
  pub typography: Typography,
  pub spacing: Spacing,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_theme_json() {
    let json = "{\"name\": \"default-dark\", \"version\": 1, \"colors\": {\"background\": \"#0F1115\", \"surface\": \"#1A1E24\", \"text_primary\": \"#FFFFFF\", \"text_secondary\": \"#9AA4B2\", \"accent\": \"#4FA3FF\", \"good\": \"#4CD964\", \"warn\": \"#FFC857\", \"critical\": \"#FF4D4D\"}, \"typography\": {\"primary_font\": \"Inter\", \"numeric_font\": \"JetBrains Mono\"}, \"spacing\": {\"grid_spacing\": 8, \"widget_padding\": 12}}";

    let theme: ThemeDocument = serde_json::from_str(json).expect("Failed to parse JSON");
    assert_eq!(theme.name, "default-dark");
    assert_eq!(theme.version, 1);
    assert_eq!(theme.colors.background, "#0F1115");
    assert_eq!(theme.typography.primary_font, "Inter");
    assert_eq!(theme.spacing.grid_spacing, 8);
  }

  #[test]
  fn test_theme_toml() {
    let toml_content = "name = \"default-dark\"\nversion = 1\n\n[colors]\nbackground = \"#0F1115\"\nsurface = \"#1A1E24\"\ntext_primary = \"#FFFFFF\"\ntext_secondary = \"#9AA4B2\"\naccent = \"#4FA3FF\"\ngood = \"#4CD964\"\nwarn = \"#FFC857\"\ncritical = \"#FF4D4D\"\n\n[typography]\nprimary_font = \"Inter\"\nnumeric_font = \"JetBrains Mono\"\n\n[spacing]\ngrid_spacing = 8\nwidget_padding = 12";

    let theme: ThemeDocument = toml::from_str(toml_content).expect("Failed to parse TOML");
    assert_eq!(theme.name, "default-dark");
    assert_eq!(theme.version, 1);
    assert_eq!(theme.colors.background, "#0F1115");
    assert_eq!(theme.typography.primary_font, "Inter");
    assert_eq!(theme.spacing.grid_spacing, 8);
  }

  #[test]
  fn test_theme_roundtrip_json() {
    let theme = ThemeDocument {
      name: "test-theme".to_string(),
      version: 2,
      colors: ColorRoles {
        background: "#111".to_string(),
        surface: "#222".to_string(),
        text_primary: "#fff".to_string(),
        text_secondary: "#aaa".to_string(),
        accent: "#00f".to_string(),
        good: "#0f0".to_string(),
        warn: "#ff0".to_string(),
        critical: "#f00".to_string(),
      },
      typography: Typography {
        primary_font: "Arial".to_string(),
        numeric_font: "Courier".to_string(),
      },
      spacing: Spacing {
        grid_spacing: 4,
        widget_padding: 8,
      },
    };

    let json = serde_json::to_string(&theme).expect("Failed to serialize to JSON");
    let parsed: ThemeDocument = serde_json::from_str(&json).expect("Failed to parse JSON");
    assert_eq!(theme, parsed);
  }

  #[test]
  fn test_theme_roundtrip_toml() {
    let theme = ThemeDocument {
      name: "test-theme".to_string(),
      version: 2,
      colors: ColorRoles {
        background: "#111".to_string(),
        surface: "#222".to_string(),
        text_primary: "#fff".to_string(),
        text_secondary: "#aaa".to_string(),
        accent: "#00f".to_string(),
        good: "#0f0".to_string(),
        warn: "#ff0".to_string(),
        critical: "#f00".to_string(),
      },
      typography: Typography {
        primary_font: "Arial".to_string(),
        numeric_font: "Courier".to_string(),
      },
      spacing: Spacing {
        grid_spacing: 4,
        widget_padding: 8,
      },
    };

    let toml_str = toml::to_string(&theme).expect("Failed to serialize to TOML");
    let parsed: ThemeDocument = toml::from_str(&toml_str).expect("Failed to parse TOML");
    assert_eq!(theme, parsed);
  }
}
