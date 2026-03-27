use crate::charts::customization::axis::AxisConfig;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::error::ChartError;

/// Configuration for a histogram chart.
///
/// Histograms display the distribution of a single numeric variable by
/// dividing the range of values into equal-width bins and showing the count
/// of data points in each bin.
///
/// The DataFrame passed via `source_key` must contain the raw numeric values
/// in the column named by `value_col`. Bin edges and counts are computed by
/// the Python renderer using `numpy.histogram`.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = HistogramConfig::builder()
///     .value("salary")
///     .x_label("Salary (k)")
///     .num_bins(12)
///     .build()?;
/// ```
pub struct HistogramConfig {
    /// Column name containing the raw numeric values to bin.
    pub value_col: String,
    /// Number of histogram bins. Defaults to `10` when `None`.
    pub num_bins: Option<u32>,
    /// Label displayed on the X axis (the value axis).
    pub x_label: String,
    /// Label displayed on the Y axis. Defaults to `"Count"` when `None`.
    pub y_label: Option<String>,
    /// Fill color for the bars as a hex string. Defaults to `"#4C72B0"` when `None`.
    pub color: Option<String>,
    /// Outline color for the bars. Defaults to `"white"` when `None`.
    pub line_color: Option<String>,
    /// Fill alpha (0.0 = transparent, 1.0 = opaque). Defaults to `0.7` when `None`.
    pub alpha: Option<f64>,
    /// Custom hover tooltip. Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis (value axis) display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis (count axis) display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`HistogramConfig`].
///
/// Required fields: `value_col` (via [`value`](Self::value)) and `x_label`
/// (via [`x_label`](Self::x_label)). All other fields are optional.
/// Calling [`build`](Self::build) without a required field returns
/// [`ChartError::MissingField`].
pub struct HistogramConfigBuilder {
    value_col: Option<String>,
    num_bins: Option<u32>,
    x_label: Option<String>,
    y_label: Option<String>,
    color: Option<String>,
    line_color: Option<String>,
    alpha: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl HistogramConfig {
    /// Create a new builder for a histogram configuration.
    #[must_use]
    pub fn builder() -> HistogramConfigBuilder {
        HistogramConfigBuilder {
            value_col: None,
            num_bins: None,
            x_label: None,
            y_label: None,
            color: None,
            line_color: None,
            alpha: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl HistogramConfigBuilder {
    /// Set the column name containing the raw numeric values to bin.
    #[must_use]
    pub fn value(mut self, col: &str) -> Self {
        self.value_col = Some(col.into());
        self
    }

    /// Set the number of histogram bins (default: 10).
    #[must_use]
    pub fn num_bins(mut self, n: u32) -> Self {
        self.num_bins = Some(n);
        self
    }

    /// Set the X-axis label text.
    #[must_use]
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set the Y-axis label text (default: `"Count"`).
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set the fill color for the bars as a hex string (e.g. `"#e74c3c"`).
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the outline color for the bars (e.g. `"#333333"`).
    #[must_use]
    pub fn line_color(mut self, color: &str) -> Self {
        self.line_color = Some(color.into());
        self
    }

    /// Set the fill alpha (0.0 = transparent, 1.0 = opaque).
    #[must_use]
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
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

    /// Configure the Y axis (count axis) appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if `value_col` or `x_label` was not set.
    pub fn build(self) -> Result<HistogramConfig, ChartError> {
        Ok(HistogramConfig {
            value_col: self
                .value_col
                .ok_or(ChartError::MissingField("value_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            num_bins: self.num_bins,
            y_label: self.y_label,
            color: self.color,
            line_color: self.line_color,
            alpha: self.alpha,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_minimal() {
        let config = HistogramConfig::builder()
            .value("salary")
            .x_label("Salary (k)")
            .build()
            .unwrap();
        assert_eq!(config.value_col, "salary");
        assert_eq!(config.x_label, "Salary (k)");
        assert!(config.num_bins.is_none());
        assert!(config.y_label.is_none());
        assert!(config.color.is_none());
        assert!(config.line_color.is_none());
        assert!(config.alpha.is_none());
        assert!(config.tooltips.is_none());
        assert!(config.x_axis.is_none());
        assert!(config.y_axis.is_none());
    }

    #[test]
    fn build_all_fields() {
        let config = HistogramConfig::builder()
            .value("age")
            .x_label("Age")
            .y_label("Frequency")
            .num_bins(20)
            .color("#e74c3c")
            .line_color("#333333")
            .alpha(0.85)
            .build()
            .unwrap();
        assert_eq!(config.value_col, "age");
        assert_eq!(config.x_label, "Age");
        assert_eq!(config.y_label.as_deref(), Some("Frequency"));
        assert_eq!(config.num_bins, Some(20));
        assert_eq!(config.color.as_deref(), Some("#e74c3c"));
        assert_eq!(config.line_color.as_deref(), Some("#333333"));
        assert_eq!(config.alpha, Some(0.85));
    }

    #[test]
    fn missing_value_col_returns_error() {
        assert!(matches!(
            HistogramConfig::builder().x_label("Value").build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn missing_x_label_returns_error() {
        assert!(matches!(
            HistogramConfig::builder().value("score").build(),
            Err(ChartError::MissingField("x_label"))
        ));
    }
}
