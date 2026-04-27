use crate::charts::customization::axis::AxisConfig;
use crate::charts::customization::marker::MarkerType;
use crate::charts::customization::palette::PaletteSpec;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::error::ChartError;

/// Configuration for a bubble plot.
///
/// A bubble plot is a scatter plot where marker size is driven by a data
/// column. Optionally a categorical column can drive color via [`PaletteSpec`].
///
/// Raw `size_col` values are mapped to a pixel range (`size_min`..`size_max`,
/// defaults `8.0`..`40.0`) with a `sqrt` transform so perceived area scales
/// linearly with the underlying value. Non-positive values clamp to `size_min`.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = BubbleConfig::builder()
///     .x("revenue")
///     .y("profit")
///     .size("headcount")
///     .color_by("region")
///     .x_label("Revenue (k)")
///     .y_label("Profit (k)")
///     .palette(PaletteSpec::Named("Category10".into()))
///     .build()?;
/// ```
pub struct BubbleConfig {
    /// Column name for the X-axis values.
    pub x_col: String,
    /// Column name for the Y-axis values.
    pub y_col: String,
    /// Column name whose numeric values drive marker size.
    pub size_col: String,
    /// Label displayed on the X axis.
    pub x_label: String,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Categorical column driving per-bubble fill color. When `None`,
    /// all bubbles share [`color`](Self::color).
    pub color_col: Option<String>,
    /// Palette used when [`color_col`](Self::color_col) is set.
    pub palette: Option<PaletteSpec>,
    /// Fallback fill color when [`color_col`](Self::color_col) is `None`.
    /// Defaults to `"#4C72B0"`.
    pub color: Option<String>,
    /// Marker shape. Defaults to [`MarkerType::Circle`] when `None`.
    pub marker: Option<MarkerType>,
    /// Minimum marker radius in screen units. Defaults to `8.0`.
    pub size_min: Option<f64>,
    /// Maximum marker radius in screen units. Defaults to `40.0`.
    pub size_max: Option<f64>,
    /// Fill alpha (0.0 = transparent, 1.0 = opaque). Defaults to `0.6`.
    pub alpha: Option<f64>,
    /// Custom hover tooltip. Defaults to x/y/size/color columns when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
    /// Whether to render a legend when [`color_col`](Self::color_col) is set.
    /// Defaults to `true`.
    pub show_legend: Option<bool>,
}

/// Builder for [`BubbleConfig`].
pub struct BubbleConfigBuilder {
    x_col: Option<String>,
    y_col: Option<String>,
    size_col: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
    color_col: Option<String>,
    palette: Option<PaletteSpec>,
    color: Option<String>,
    marker: Option<MarkerType>,
    size_min: Option<f64>,
    size_max: Option<f64>,
    alpha: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    show_legend: Option<bool>,
}

impl BubbleConfig {
    #[must_use]
    pub fn builder() -> BubbleConfigBuilder {
        BubbleConfigBuilder {
            x_col: None,
            y_col: None,
            size_col: None,
            x_label: None,
            y_label: None,
            color_col: None,
            palette: None,
            color: None,
            marker: None,
            size_min: None,
            size_max: None,
            alpha: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
            show_legend: None,
        }
    }
}

impl BubbleConfigBuilder {
    #[must_use]
    pub fn x(mut self, col: &str) -> Self {
        self.x_col = Some(col.into());
        self
    }
    #[must_use]
    pub fn y(mut self, col: &str) -> Self {
        self.y_col = Some(col.into());
        self
    }
    #[must_use]
    pub fn size(mut self, col: &str) -> Self {
        self.size_col = Some(col.into());
        self
    }
    #[must_use]
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.into());
        self
    }
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }
    #[must_use]
    pub fn color_by(mut self, col: &str) -> Self {
        self.color_col = Some(col.into());
        self
    }
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }
    #[must_use]
    pub fn marker(mut self, marker: MarkerType) -> Self {
        self.marker = Some(marker);
        self
    }
    #[must_use]
    pub fn size_range(mut self, min: f64, max: f64) -> Self {
        self.size_min = Some(min);
        self.size_max = Some(max);
        self
    }
    #[must_use]
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
        self
    }
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }
    #[must_use]
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = Some(show);
        self
    }

    /// Build the config.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<BubbleConfig, ChartError> {
        Ok(BubbleConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            y_col: self.y_col.ok_or(ChartError::MissingField("y_col"))?,
            size_col: self.size_col.ok_or(ChartError::MissingField("size_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            color_col: self.color_col,
            palette: self.palette,
            color: self.color,
            marker: self.marker,
            size_min: self.size_min,
            size_max: self.size_max,
            alpha: self.alpha,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
            show_legend: self.show_legend,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_x_col() {
        assert!(matches!(
            BubbleConfig::builder()
                .y("y").size("s").x_label("X").y_label("Y").build(),
            Err(ChartError::MissingField("x_col"))
        ));
    }

    #[test]
    fn missing_size_col() {
        assert!(matches!(
            BubbleConfig::builder()
                .x("x").y("y").x_label("X").y_label("Y").build(),
            Err(ChartError::MissingField("size_col"))
        ));
    }

    #[test]
    fn build_success() {
        let cfg = BubbleConfig::builder()
            .x("rev").y("prof").size("head")
            .x_label("Revenue").y_label("Profit")
            .build().unwrap();
        assert_eq!(cfg.x_col, "rev");
        assert_eq!(cfg.size_col, "head");
        assert!(cfg.color_col.is_none());
        assert!(cfg.size_min.is_none());
    }

    #[test]
    fn optional_fields() {
        let cfg = BubbleConfig::builder()
            .x("x").y("y").size("s").x_label("X").y_label("Y")
            .color_by("region")
            .palette(PaletteSpec::Named("Category10".into()))
            .color("#ff0000")
            .marker(MarkerType::Diamond)
            .size_range(5.0, 50.0)
            .alpha(0.4)
            .show_legend(false)
            .build().unwrap();
        assert_eq!(cfg.color_col.as_deref(), Some("region"));
        assert!(matches!(cfg.palette, Some(PaletteSpec::Named(ref s)) if s == "Category10"));
        assert_eq!(cfg.color.as_deref(), Some("#ff0000"));
        assert_eq!(cfg.marker, Some(MarkerType::Diamond));
        assert_eq!(cfg.size_min, Some(5.0));
        assert_eq!(cfg.size_max, Some(50.0));
        assert_eq!(cfg.alpha, Some(0.4));
        assert_eq!(cfg.show_legend, Some(false));
    }
}
