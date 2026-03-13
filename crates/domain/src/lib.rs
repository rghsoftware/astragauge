pub mod panel;
pub mod provider;
pub mod sensor;
pub mod theme;
pub mod validation;
pub mod widget;

pub use panel::{GridConfig, PanelDocument, WidgetPlacement};
pub use provider::{ProviderCapabilities, ProviderManifest, SensorCategories};
pub use sensor::{SensorDescriptor, SensorSample};
pub use theme::{ColorRoles, Spacing, ThemeDocument, Typography};
pub use validation::{DomainError, SensorId};
pub use widget::{
  BindingTarget, BindingValueType, Capabilities, MockKind, Preview, Property, PropertyType,
  ResizeMode, Sizing, Theming, Validation, WidgetManifest,
};

#[cfg(test)]
mod reexport_tests {
  #[test]
  fn test_all_types_accessible_from_root() {
    use crate::{
      BindingTarget, BindingValueType, Capabilities, ColorRoles, DomainError, GridConfig, MockKind,
      PanelDocument, Preview, Property, PropertyType, ProviderCapabilities, ProviderManifest,
      ResizeMode, SensorCategories, SensorDescriptor, SensorId, SensorSample, Sizing, Spacing,
      ThemeDocument, Theming, Typography, Validation, WidgetManifest, WidgetPlacement,
    };

    let _: Option<SensorId> = None;
    let _: Option<DomainError> = None;
    let _: Option<SensorDescriptor> = None;
    let _: Option<SensorSample> = None;
    let _: Option<GridConfig> = None;
    let _: Option<WidgetPlacement> = None;
    let _: Option<PanelDocument> = None;
    let _: Option<ColorRoles> = None;
    let _: Option<Typography> = None;
    let _: Option<Spacing> = None;
    let _: Option<ThemeDocument> = None;
    let _: Option<ResizeMode> = None;
    let _: Option<PropertyType> = None;
    let _: Option<BindingValueType> = None;
    let _: Option<MockKind> = None;
    let _: Option<Sizing> = None;
    let _: Option<Property> = None;
    let _: Option<BindingTarget> = None;
    let _: Option<Preview> = None;
    let _: Option<Theming> = None;
    let _: Option<Capabilities> = None;
    let _: Option<Validation> = None;
    let _: Option<WidgetManifest> = None;
    let _: Option<ProviderCapabilities> = None;
    let _: Option<SensorCategories> = None;
    let _: Option<ProviderManifest> = None;
  }
}
