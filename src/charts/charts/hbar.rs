use crate::error::ChartError;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::charts::customization::axis::AxisConfig;

/// Configuration for a horizontal bar chart.
///
/// Horizontal bar charts are useful for displaying ranked or categorical data
/// where the category labels are long strings. The bars extend horizontally
/// from left to right, with categories listed vertically.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = HBarConfig::builder()
///     .category("department")
///     .value("headcount")
///     .x_label("Employees")
///     .build()?;
/// ```
pub struct HBarConfig {
    /// Column name for the categorical axis (displayed vertically).
    pub category_col: String,
    /// Column name for the numeric values (bar lengths).
    pub value_col: String,
    /// Label displayed on the X axis (the value axis for horizontal bars).
    pub x_label: String,
    /// Fill color for the bars as a hex string (e.g. `"#e74c3c"`).
    /// Defaults to `"#4C72B0"` when `None`.
    pub color: Option<String>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis (value axis) display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis (category axis) display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`HBarConfig`].
///
/// All fields are required. Calling [`build`](HBarConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct HBarConfigBuilder {
    category_col: Option<String>,
    value_col: Option<String>,
    x_label: Option<String>,
    color: Option<String>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl HBarConfig {
    /// Create a new builder for a horizontal bar chart configuration.
    #[must_use]
    pub fn builder() -> HBarConfigBuilder {
        HBarConfigBuilder {
            category_col: None,
            value_col: None,
            x_label: None,
            color: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl HBarConfigBuilder {
    /// Set the category column name.
    #[must_use]
    pub fn category(mut self, col: &str) -> Self {
        self.category_col = Some(col.into());
        self
    }
    /// Set the numeric value column name.
    #[must_use]
    pub fn value(mut self, col: &str) -> Self {
        self.value_col = Some(col.into());
        self
    }
    /// Set the X-axis label text.
    #[must_use]
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.into());
        self
    }
    /// Set the fill color for the bars as a hex string (e.g. `"#e74c3c"`).
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }
    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    /// Configure the X axis (value axis) appearance.
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    /// Configure the Y axis (category axis) appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<HBarConfig, ChartError> {
        Ok(HBarConfig {
            category_col: self
                .category_col
                .ok_or(ChartError::MissingField("category_col"))?,
            value_col: self
                .value_col
                .ok_or(ChartError::MissingField("value_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            color: self.color,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::customization::tooltip::{TooltipSpec, TooltipFormat};
    use crate::charts::customization::axis::AxisConfig;

    // ── HBarConfig builder ────────────────────────────────────────────────────

    #[test]
    fn hbar_missing_category_col() {
        assert!(matches!(
            HBarConfig::builder().value("v").x_label("X").build(),
            Err(ChartError::MissingField("category_col"))
        ));
    }

    #[test]
    fn hbar_missing_value_col() {
        assert!(matches!(
            HBarConfig::builder().category("c").x_label("X").build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn hbar_missing_x_label() {
        assert!(matches!(
            HBarConfig::builder().category("c").value("v").build(),
            Err(ChartError::MissingField("x_label"))
        ));
    }

    #[test]
    fn hbar_build_success() {
        let cfg = HBarConfig::builder()
            .category("dept")
            .value("headcount")
            .x_label("Employees")
            .build()
            .unwrap();
        assert_eq!(cfg.category_col, "dept");
        assert_eq!(cfg.value_col, "headcount");
        assert_eq!(cfg.x_label, "Employees");
    }

    // ── HBarConfig optional fields ────────────────────────────────────────────

    #[test]
    fn hbar_optional_fields_default_none() {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .build().unwrap();
        assert!(cfg.color.is_none());
        assert!(cfg.tooltips.is_none());
        assert!(cfg.x_axis.is_none());
        assert!(cfg.y_axis.is_none());
    }

    #[test]
    fn hbar_with_color() {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .color("#e74c3c")
            .build().unwrap();
        assert_eq!(cfg.color.as_deref(), Some("#e74c3c"));
    }

    #[test]
    fn hbar_with_tooltips() {
        let tt = TooltipSpec::builder()
            .field("c", "Cat", TooltipFormat::Text)
            .field("v", "Val", TooltipFormat::Number(None))
            .build();
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .tooltips(tt)
            .build().unwrap();
        let fields = &cfg.tooltips.unwrap().fields;
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].column, "c");
        assert_eq!(fields[1].column, "v");
    }

    #[test]
    fn hbar_with_x_axis_bounds() {
        let ax = AxisConfig::builder().bounds(0.0, 100.0).build();
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .x_axis(ax)
            .build().unwrap();
        assert_eq!(cfg.x_axis.as_ref().unwrap().bounds_min, Some(0.0));
        assert_eq!(cfg.x_axis.as_ref().unwrap().bounds_max, Some(100.0));
    }

    #[test]
    fn hbar_with_y_axis() {
        let ax = AxisConfig::builder().label_rotation(30.0).build();
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .y_axis(ax)
            .build().unwrap();
        assert_eq!(cfg.y_axis.as_ref().unwrap().label_rotation, Some(30.0));
    }
}
