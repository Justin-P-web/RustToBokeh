use crate::error::ChartError;
use crate::charts::customization::palette::PaletteSpec;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::charts::customization::axis::AxisConfig;

/// Configuration for a grouped bar chart.
///
/// Grouped bar charts display vertical bars organized by a categorical X axis,
/// with bars within each group distinguished by a grouping column. The
/// `DataFrame` must contain three columns: one for the X-axis categories, one
/// for the group within each category, and one for the numeric value.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = GroupedBarConfig::builder()
///     .x("month")           // X-axis category column
///     .group("product")     // Grouping column (one bar per group value)
///     .value("revenue")     // Numeric value column (bar height)
///     .y_label("Revenue (k)")
///     .build()?;
/// ```
pub struct GroupedBarConfig {
    /// Column name for the X-axis categories (e.g. `"month"`, `"quarter"`).
    pub x_col: String,
    /// Column name for the grouping variable within each X category.
    pub group_col: String,
    /// Column name for the numeric values (bar heights).
    pub value_col: String,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Color palette for the group bars.  Defaults to the built-in seaborn
    /// color cycle when `None`.
    pub palette: Option<PaletteSpec>,
    /// Width of each bar as a fraction of the available slot (0.0–1.0).
    /// Defaults to `0.9` when `None`.
    pub bar_width: Option<f64>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`GroupedBarConfig`].
///
/// All fields are required. Calling [`build`](GroupedBarConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct GroupedBarConfigBuilder {
    x_col: Option<String>,
    group_col: Option<String>,
    value_col: Option<String>,
    y_label: Option<String>,
    palette: Option<PaletteSpec>,
    bar_width: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl GroupedBarConfig {
    /// Create a new builder for a grouped bar chart configuration.
    #[must_use]
    pub fn builder() -> GroupedBarConfigBuilder {
        GroupedBarConfigBuilder {
            x_col: None,
            group_col: None,
            value_col: None,
            y_label: None,
            palette: None,
            bar_width: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl GroupedBarConfigBuilder {
    /// Set the X-axis category column name.
    #[must_use]
    pub fn x(mut self, col: &str) -> Self {
        self.x_col = Some(col.into());
        self
    }
    /// Set the grouping column name.
    #[must_use]
    pub fn group(mut self, col: &str) -> Self {
        self.group_col = Some(col.into());
        self
    }
    /// Set the numeric value column name.
    #[must_use]
    pub fn value(mut self, col: &str) -> Self {
        self.value_col = Some(col.into());
        self
    }
    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }
    /// Set the color palette for the group bars.
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }
    /// Set the bar width as a fraction of the available slot width (0.0–1.0).
    #[must_use]
    pub fn bar_width(mut self, width: f64) -> Self {
        self.bar_width = Some(width);
        self
    }
    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    /// Configure the X axis appearance.
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    /// Configure the Y axis appearance.
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
    pub fn build(self) -> Result<GroupedBarConfig, ChartError> {
        Ok(GroupedBarConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            group_col: self
                .group_col
                .ok_or(ChartError::MissingField("group_col"))?,
            value_col: self
                .value_col
                .ok_or(ChartError::MissingField("value_col"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            palette: self.palette,
            bar_width: self.bar_width,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::customization::palette::PaletteSpec;
    use crate::charts::customization::tooltip::{TooltipSpec, TooltipFormat};
    use crate::charts::customization::axis::AxisConfig;

    // ── GroupedBarConfig builder ──────────────────────────────────────────────

    #[test]
    fn grouped_bar_missing_x_col() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .group("g")
                .value("v")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("x_col"))
        ));
    }

    #[test]
    fn grouped_bar_missing_group_col() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .x("x")
                .value("v")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("group_col"))
        ));
    }

    #[test]
    fn grouped_bar_missing_value_col() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .x("x")
                .group("g")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn grouped_bar_missing_y_label() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .x("x")
                .group("g")
                .value("v")
                .build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    #[test]
    fn grouped_bar_build_success() {
        let cfg = GroupedBarConfig::builder()
            .x("month")
            .group("category")
            .value("revenue")
            .y_label("USD")
            .build()
            .unwrap();
        assert_eq!(cfg.x_col, "month");
        assert_eq!(cfg.group_col, "category");
        assert_eq!(cfg.value_col, "revenue");
        assert_eq!(cfg.y_label, "USD");
    }

    // ── GroupedBarConfig optional fields ──────────────────────────────────────

    #[test]
    fn grouped_bar_optional_fields_default_none() {
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .build().unwrap();
        assert!(cfg.palette.is_none());
        assert!(cfg.bar_width.is_none());
        assert!(cfg.tooltips.is_none());
        assert!(cfg.x_axis.is_none());
        assert!(cfg.y_axis.is_none());
    }

    #[test]
    fn grouped_bar_with_named_palette() {
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .palette(PaletteSpec::Named("Viridis256".into()))
            .build().unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Named(ref n)) if n == "Viridis256"));
    }

    #[test]
    fn grouped_bar_with_custom_palette() {
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .palette(PaletteSpec::Custom(vec!["#ff0000".into()]))
            .build().unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Custom(_))));
    }

    #[test]
    fn grouped_bar_with_bar_width() {
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .bar_width(0.6)
            .build().unwrap();
        assert_eq!(cfg.bar_width, Some(0.6));
    }

    #[test]
    fn grouped_bar_with_tooltips() {
        let tt = TooltipSpec::builder()
            .field("x", "X", TooltipFormat::Text)
            .build();
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .tooltips(tt)
            .build().unwrap();
        assert!(cfg.tooltips.is_some());
        assert_eq!(cfg.tooltips.unwrap().fields.len(), 1);
    }

    #[test]
    fn grouped_bar_with_x_axis() {
        let ax = AxisConfig::builder().tick_format("$0").build();
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .x_axis(ax)
            .build().unwrap();
        assert!(cfg.x_axis.is_some());
        assert_eq!(cfg.x_axis.unwrap().tick_format.as_deref(), Some("$0"));
    }

    #[test]
    fn grouped_bar_with_y_axis() {
        let ax = AxisConfig::builder().range(0.0, 200.0).build();
        let cfg = GroupedBarConfig::builder()
            .x("x").group("g").value("v").y_label("Y")
            .y_axis(ax)
            .build().unwrap();
        assert_eq!(cfg.y_axis.as_ref().unwrap().start, Some(0.0));
        assert_eq!(cfg.y_axis.as_ref().unwrap().end, Some(200.0));
    }
}
