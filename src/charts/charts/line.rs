use crate::error::ChartError;
use crate::charts::customization::palette::PaletteSpec;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::charts::customization::axis::AxisConfig;

/// Configuration for a multi-line chart.
///
/// Line charts plot one or more numeric series against a shared X axis. Each
/// entry in `y_cols` becomes a separate line rendered with a distinct color.
/// When multiple line or scatter charts on the same page share the same
/// `source_key`, they share a single Bokeh `ColumnDataSource`, enabling linked
/// hover and selection across charts.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = LineConfig::builder()
///     .x("month")
///     .y_cols(&["revenue", "expenses", "profit"])
///     .y_label("USD (k)")
///     .build()?;
/// ```
pub struct LineConfig {
    /// Column name for the X axis (typically a time or category column).
    pub x_col: String,
    /// Column names for the Y-axis series. Each column produces one line.
    pub y_cols: Vec<String>,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Color palette for the lines.  Defaults to the built-in seaborn color
    /// cycle when `None`.
    pub palette: Option<PaletteSpec>,
    /// Stroke width of the lines in screen units.  Defaults to `2.5` when `None`.
    pub line_width: Option<f64>,
    /// Size of the scatter markers drawn at each data point.
    /// Defaults to `7` when `None`.
    pub point_size: Option<f64>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`LineConfig`].
///
/// All fields are required. Calling [`build`](LineConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct LineConfigBuilder {
    x_col: Option<String>,
    y_cols: Option<Vec<String>>,
    y_label: Option<String>,
    palette: Option<PaletteSpec>,
    line_width: Option<f64>,
    point_size: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl LineConfig {
    /// Create a new builder for a line chart configuration.
    #[must_use]
    pub fn builder() -> LineConfigBuilder {
        LineConfigBuilder {
            x_col: None,
            y_cols: None,
            y_label: None,
            palette: None,
            line_width: None,
            point_size: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl LineConfigBuilder {
    /// Set the X-axis column name.
    #[must_use]
    pub fn x(mut self, col: &str) -> Self {
        self.x_col = Some(col.into());
        self
    }
    /// Set the Y-axis column names. Each column becomes a separate line.
    #[must_use]
    pub fn y_cols(mut self, cols: &[&str]) -> Self {
        self.y_cols = Some(cols.iter().map(|&s| s.into()).collect());
        self
    }
    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }
    /// Set the color palette for the lines.
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }
    /// Set the stroke width of each line in screen units.
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = Some(width);
        self
    }
    /// Set the size of the scatter markers drawn at each data point.
    #[must_use]
    pub fn point_size(mut self, size: f64) -> Self {
        self.point_size = Some(size);
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
    pub fn build(self) -> Result<LineConfig, ChartError> {
        Ok(LineConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            y_cols: self.y_cols.ok_or(ChartError::MissingField("y_cols"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            palette: self.palette,
            line_width: self.line_width,
            point_size: self.point_size,
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

    // ── LineConfig builder ────────────────────────────────────────────────────

    #[test]
    fn line_missing_x_col() {
        assert!(matches!(
            LineConfig::builder().y_cols(&["a"]).y_label("Y").build(),
            Err(ChartError::MissingField("x_col"))
        ));
    }

    #[test]
    fn line_missing_y_cols() {
        assert!(matches!(
            LineConfig::builder().x("x").y_label("Y").build(),
            Err(ChartError::MissingField("y_cols"))
        ));
    }

    #[test]
    fn line_missing_y_label() {
        assert!(matches!(
            LineConfig::builder().x("x").y_cols(&["a"]).build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    #[test]
    fn line_build_success() {
        let cfg = LineConfig::builder()
            .x("month")
            .y_cols(&["rev", "exp"])
            .y_label("USD")
            .build()
            .unwrap();
        assert_eq!(cfg.x_col, "month");
        assert_eq!(cfg.y_cols, vec!["rev".to_string(), "exp".to_string()]);
        assert_eq!(cfg.y_label, "USD");
    }

    // ── LineConfig optional fields ────────────────────────────────────────────

    #[test]
    fn line_optional_fields_default_none() {
        let cfg = LineConfig::builder()
            .x("x").y_cols(&["a"]).y_label("Y")
            .build().unwrap();
        assert!(cfg.palette.is_none());
        assert!(cfg.line_width.is_none());
        assert!(cfg.point_size.is_none());
        assert!(cfg.tooltips.is_none());
        assert!(cfg.x_axis.is_none());
        assert!(cfg.y_axis.is_none());
    }

    #[test]
    fn line_with_palette() {
        let cfg = LineConfig::builder()
            .x("x").y_cols(&["a"]).y_label("Y")
            .palette(PaletteSpec::Named("Magma256".into()))
            .build().unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Named(_))));
    }

    #[test]
    fn line_with_line_width() {
        let cfg = LineConfig::builder()
            .x("x").y_cols(&["a"]).y_label("Y")
            .line_width(4.0)
            .build().unwrap();
        assert_eq!(cfg.line_width, Some(4.0));
    }

    #[test]
    fn line_with_point_size() {
        let cfg = LineConfig::builder()
            .x("x").y_cols(&["a"]).y_label("Y")
            .point_size(12.0)
            .build().unwrap();
        assert_eq!(cfg.point_size, Some(12.0));
    }

    #[test]
    fn line_with_tooltips() {
        let tt = TooltipSpec::builder()
            .field("a", "A", TooltipFormat::Number(Some(1)))
            .build();
        let cfg = LineConfig::builder()
            .x("x").y_cols(&["a"]).y_label("Y")
            .tooltips(tt)
            .build().unwrap();
        assert!(cfg.tooltips.is_some());
    }

    #[test]
    fn line_with_y_axis() {
        let ax = AxisConfig::builder().show_grid(false).build();
        let cfg = LineConfig::builder()
            .x("x").y_cols(&["a"]).y_label("Y")
            .y_axis(ax)
            .build().unwrap();
        assert!(!cfg.y_axis.as_ref().unwrap().show_grid);
    }
}
