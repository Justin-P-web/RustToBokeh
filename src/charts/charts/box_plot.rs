use crate::charts::customization::axis::AxisConfig;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::error::ChartError;

/// Configuration for a box plot (box-and-whisker chart).
///
/// Box plots display the distribution of a numeric variable across categories,
/// showing the interquartile range (IQR) as a box, whiskers extending to the
/// most extreme non-outlier observations, and the median as a line inside the box.
///
/// The chart expects a pre-computed statistics `DataFrame` produced by
/// [`compute_box_stats`](crate::compute_box_stats), which provides `category`,
/// `q1`, `q2`, `q3`, `lower`, and `upper` columns. Column names can be
/// customised if you bring your own pre-computed data.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// // Prepare data:
/// let raw = df![
///     "department" => ["Eng", "Sales", "Eng", "Sales"],
///     "salary"     => [95.0f64, 70.0, 105.0, 80.0],
/// ].unwrap();
/// let mut stats = compute_box_stats(&raw, "department", "salary")?;
/// dash.add_df("salary_box", &mut stats)?;
///
/// // Define chart:
/// let config = BoxPlotConfig::builder()
///     .category("category")
///     .q1("q1")
///     .q2("q2")
///     .q3("q3")
///     .lower("lower")
///     .upper("upper")
///     .y_label("Salary (k)")
///     .build()?;
/// ```
pub struct BoxPlotConfig {
    /// Column name for the category labels (X axis).
    pub category_col: String,
    /// Column name for the first quartile (25th percentile).
    pub q1_col: String,
    /// Column name for the median (50th percentile).
    pub q2_col: String,
    /// Column name for the third quartile (75th percentile).
    pub q3_col: String,
    /// Column name for the lower whisker endpoint (most extreme non-outlier below Q1).
    pub lower_col: String,
    /// Column name for the upper whisker endpoint (most extreme non-outlier above Q3).
    pub upper_col: String,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Fill color for the IQR boxes as a hex string. Defaults to `"#4C72B0"`.
    pub color: Option<String>,
    /// Fill alpha (0.0 = transparent, 1.0 = opaque). Defaults to `0.7`.
    pub alpha: Option<f64>,
    /// Custom hover tooltip. When `None`, a default is generated showing all five statistics.
    pub tooltips: Option<TooltipSpec>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`BoxPlotConfig`].
///
/// All seven core fields are required. Calling [`build`](BoxPlotConfigBuilder::build)
/// without setting any of them returns [`ChartError::MissingField`].
pub struct BoxPlotConfigBuilder {
    category_col: Option<String>,
    q1_col: Option<String>,
    q2_col: Option<String>,
    q3_col: Option<String>,
    lower_col: Option<String>,
    upper_col: Option<String>,
    y_label: Option<String>,
    color: Option<String>,
    alpha: Option<f64>,
    tooltips: Option<TooltipSpec>,
    y_axis: Option<AxisConfig>,
}

impl BoxPlotConfig {
    /// Create a new builder for a box plot configuration.
    #[must_use]
    pub fn builder() -> BoxPlotConfigBuilder {
        BoxPlotConfigBuilder {
            category_col: None,
            q1_col: None,
            q2_col: None,
            q3_col: None,
            lower_col: None,
            upper_col: None,
            y_label: None,
            color: None,
            alpha: None,
            tooltips: None,
            y_axis: None,
        }
    }
}

impl BoxPlotConfigBuilder {
    /// Set the category column name (X axis labels).
    #[must_use]
    pub fn category(mut self, col: &str) -> Self {
        self.category_col = Some(col.into());
        self
    }

    /// Set the Q1 (25th percentile) column name.
    #[must_use]
    pub fn q1(mut self, col: &str) -> Self {
        self.q1_col = Some(col.into());
        self
    }

    /// Set the Q2 (median) column name.
    #[must_use]
    pub fn q2(mut self, col: &str) -> Self {
        self.q2_col = Some(col.into());
        self
    }

    /// Set the Q3 (75th percentile) column name.
    #[must_use]
    pub fn q3(mut self, col: &str) -> Self {
        self.q3_col = Some(col.into());
        self
    }

    /// Set the lower whisker endpoint column name.
    #[must_use]
    pub fn lower(mut self, col: &str) -> Self {
        self.lower_col = Some(col.into());
        self
    }

    /// Set the upper whisker endpoint column name.
    #[must_use]
    pub fn upper(mut self, col: &str) -> Self {
        self.upper_col = Some(col.into());
        self
    }

    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set the fill color for the IQR boxes as a hex string.
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
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
    pub fn build(self) -> Result<BoxPlotConfig, ChartError> {
        Ok(BoxPlotConfig {
            category_col: self.category_col.ok_or(ChartError::MissingField("category_col"))?,
            q1_col:       self.q1_col.ok_or(ChartError::MissingField("q1_col"))?,
            q2_col:       self.q2_col.ok_or(ChartError::MissingField("q2_col"))?,
            q3_col:       self.q3_col.ok_or(ChartError::MissingField("q3_col"))?,
            lower_col:    self.lower_col.ok_or(ChartError::MissingField("lower_col"))?,
            upper_col:    self.upper_col.ok_or(ChartError::MissingField("upper_col"))?,
            y_label:      self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            color:        self.color,
            alpha:        self.alpha,
            tooltips:     self.tooltips,
            y_axis:       self.y_axis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::customization::axis::AxisConfig;

    fn minimal() -> BoxPlotConfig {
        BoxPlotConfig::builder()
            .category("category")
            .q1("q1")
            .q2("q2")
            .q3("q3")
            .lower("lower")
            .upper("upper")
            .y_label("Value")
            .build()
            .unwrap()
    }

    // ── Required field validation ─────────────────────────────────────────────

    #[test]
    fn missing_category_col() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .q1("q1").q2("q2").q3("q3")
                .lower("lower").upper("upper")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("category_col"))
        ));
    }

    #[test]
    fn missing_q1_col() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .category("cat").q2("q2").q3("q3")
                .lower("lower").upper("upper")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("q1_col"))
        ));
    }

    #[test]
    fn missing_q2_col() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .category("cat").q1("q1").q3("q3")
                .lower("lower").upper("upper")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("q2_col"))
        ));
    }

    #[test]
    fn missing_q3_col() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .category("cat").q1("q1").q2("q2")
                .lower("lower").upper("upper")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("q3_col"))
        ));
    }

    #[test]
    fn missing_lower_col() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .category("cat").q1("q1").q2("q2").q3("q3")
                .upper("upper")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("lower_col"))
        ));
    }

    #[test]
    fn missing_upper_col() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .category("cat").q1("q1").q2("q2").q3("q3")
                .lower("lower")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("upper_col"))
        ));
    }

    #[test]
    fn missing_y_label() {
        assert!(matches!(
            BoxPlotConfig::builder()
                .category("cat").q1("q1").q2("q2").q3("q3")
                .lower("lower").upper("upper")
                .build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    // ── Build success ─────────────────────────────────────────────────────────

    #[test]
    fn build_success() {
        let cfg = minimal();
        assert_eq!(cfg.category_col, "category");
        assert_eq!(cfg.q1_col, "q1");
        assert_eq!(cfg.q2_col, "q2");
        assert_eq!(cfg.q3_col, "q3");
        assert_eq!(cfg.lower_col, "lower");
        assert_eq!(cfg.upper_col, "upper");
        assert_eq!(cfg.y_label, "Value");
    }

    // ── Optional fields default to None ──────────────────────────────────────

    #[test]
    fn optional_fields_default_none() {
        let cfg = minimal();
        assert!(cfg.color.is_none());
        assert!(cfg.alpha.is_none());
        assert!(cfg.tooltips.is_none());
        assert!(cfg.y_axis.is_none());
    }

    // ── Optional field setters ────────────────────────────────────────────────

    #[test]
    fn with_color() {
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Y")
            .color("#2ecc71")
            .build()
            .unwrap();
        assert_eq!(cfg.color.as_deref(), Some("#2ecc71"));
    }

    #[test]
    fn with_alpha() {
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Y")
            .alpha(0.5)
            .build()
            .unwrap();
        assert_eq!(cfg.alpha, Some(0.5));
    }

    #[test]
    fn with_y_axis() {
        let ax = AxisConfig::builder().tick_format("0.0").show_grid(false).build();
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Y")
            .y_axis(ax)
            .build()
            .unwrap();
        let y = cfg.y_axis.as_ref().unwrap();
        assert_eq!(y.tick_format.as_deref(), Some("0.0"));
        assert!(!y.show_grid);
    }

    #[test]
    fn custom_column_names() {
        let cfg = BoxPlotConfig::builder()
            .category("dept")
            .q1("p25").q2("median").q3("p75")
            .lower("fence_lo").upper("fence_hi")
            .y_label("Score")
            .build()
            .unwrap();
        assert_eq!(cfg.category_col, "dept");
        assert_eq!(cfg.q1_col, "p25");
        assert_eq!(cfg.q2_col, "median");
        assert_eq!(cfg.q3_col, "p75");
        assert_eq!(cfg.lower_col, "fence_lo");
        assert_eq!(cfg.upper_col, "fence_hi");
        assert_eq!(cfg.y_label, "Score");
    }
}
