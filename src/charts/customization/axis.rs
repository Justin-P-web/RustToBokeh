use super::time_scale::TimeScale;

/// Per-axis display customisation for a chart.
///
/// Controls the initial visible range, pan/zoom bounds, tick-label formatting,
/// label orientation, and grid-line visibility.  All fields are optional;
/// omitting a field preserves the Bokeh default for that property.
///
/// Build with [`AxisConfig::builder`].
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// // Dollar-formatted X axis, view 0–500, pan locked to 0–600
/// let x = AxisConfig::builder()
///     .range(0.0, 500.0)
///     .bounds(0.0, 600.0)
///     .tick_format("$0,0")
///     .build();
///
/// // Y axis with 45° label rotation and no grid lines
/// let y = AxisConfig::builder()
///     .label_rotation(45.0)
///     .show_grid(false)
///     .build();
/// ```
pub struct AxisConfig {
    /// Start of the initial visible range (`Range1d.start` in Bokeh).
    pub start: Option<f64>,
    /// End of the initial visible range (`Range1d.end` in Bokeh).
    pub end: Option<f64>,
    /// Lower pan/zoom bound (`x_range.bounds[0]` in Bokeh).
    /// Requires [`bounds_max`](AxisConfig::bounds_max) to also be set.
    pub bounds_min: Option<f64>,
    /// Upper pan/zoom bound (`x_range.bounds[1]` in Bokeh).
    /// Requires [`bounds_min`](AxisConfig::bounds_min) to also be set.
    pub bounds_max: Option<f64>,
    /// Rotation of major-tick labels in degrees (e.g. `45.0`).
    pub label_rotation: Option<f64>,
    /// [Numeral.js](http://numeraljs.com/) format string for tick labels
    /// (e.g. `"$0,0"`, `"0.0%"`, `"0.00"`).
    /// Ignored when [`time_scale`](AxisConfig::time_scale) is set.
    pub tick_format: Option<String>,
    /// Whether to draw grid lines for this axis.  Defaults to `true`.
    pub show_grid: bool,
    /// When set, configures the axis as a datetime axis using
    /// Bokeh's `DatetimeTickFormatter` at the specified resolution.
    /// The chart's x-range is automatically switched to `Range1d` for
    /// line and scatter charts.
    pub time_scale: Option<TimeScale>,
}

/// Builder for [`AxisConfig`].
pub struct AxisConfigBuilder {
    start: Option<f64>,
    end: Option<f64>,
    bounds_min: Option<f64>,
    bounds_max: Option<f64>,
    label_rotation: Option<f64>,
    tick_format: Option<String>,
    show_grid: bool,
    time_scale: Option<TimeScale>,
}

impl AxisConfig {
    /// Create a new builder for axis configuration.
    #[must_use]
    pub fn builder() -> AxisConfigBuilder {
        AxisConfigBuilder {
            start: None,
            end: None,
            bounds_min: None,
            bounds_max: None,
            label_rotation: None,
            tick_format: None,
            show_grid: true,
            time_scale: None,
        }
    }
}

impl AxisConfigBuilder {
    /// Set the initial visible range of the axis.
    ///
    /// Maps to `Range1d(start=start, end=end)` in Bokeh.
    #[must_use]
    pub fn range(mut self, start: f64, end: f64) -> Self {
        self.start = Some(start);
        self.end = Some(end);
        self
    }

    /// Set the pan/zoom bounding limits for the axis.
    ///
    /// Maps to `range.bounds = (min, max)` in Bokeh.  Both values must be
    /// supplied; the user cannot pan or zoom beyond these limits at runtime.
    #[must_use]
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        self.bounds_min = Some(min);
        self.bounds_max = Some(max);
        self
    }

    /// Set the rotation of major-tick labels in degrees.
    ///
    /// Positive values rotate the labels counter-clockwise.  `45.0` is a
    /// common choice for long category labels.
    #[must_use]
    pub fn label_rotation(mut self, degrees: f64) -> Self {
        self.label_rotation = Some(degrees);
        self
    }

    /// Set the [numeral.js](http://numeraljs.com/) format string for tick labels.
    ///
    /// Examples: `"$0,0"` (currency with commas), `"0.0%"` (percentage),
    /// `"0.00"` (fixed two decimals), `"0.0a"` (abbreviated, e.g. `"1.2k"`).
    #[must_use]
    pub fn tick_format(mut self, fmt: &str) -> Self {
        self.tick_format = Some(fmt.into());
        self
    }

    /// Control whether grid lines are drawn for this axis (default: `true`).
    #[must_use]
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Mark this axis as a datetime axis at the given time resolution.
    ///
    /// When set, Bokeh's `DatetimeTickFormatter` is applied using the format
    /// string from [`TimeScale::format_str`]. For line and scatter charts the
    /// x-range is automatically switched to `Range1d` (numeric/datetime) so
    /// Bokeh renders correct datetime tick labels.
    ///
    /// Data values must be stored as milliseconds since the Unix epoch
    /// (integers or floats).
    #[must_use]
    pub fn time_scale(mut self, scale: TimeScale) -> Self {
        self.time_scale = Some(scale);
        self
    }

    /// Consume the builder and produce an [`AxisConfig`].
    #[must_use]
    pub fn build(self) -> AxisConfig {
        AxisConfig {
            start: self.start,
            end: self.end,
            bounds_min: self.bounds_min,
            bounds_max: self.bounds_max,
            label_rotation: self.label_rotation,
            tick_format: self.tick_format,
            show_grid: self.show_grid,
            time_scale: self.time_scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AxisConfig / AxisConfigBuilder ────────────────────────────────────────

    #[test]
    fn axis_config_defaults_all_none_show_grid_true() {
        let ax = AxisConfig::builder().build();
        assert!(ax.start.is_none());
        assert!(ax.end.is_none());
        assert!(ax.bounds_min.is_none());
        assert!(ax.bounds_max.is_none());
        assert!(ax.label_rotation.is_none());
        assert!(ax.tick_format.is_none());
        assert!(ax.show_grid);
    }

    #[test]
    fn axis_config_range_sets_start_and_end() {
        let ax = AxisConfig::builder().range(0.0, 100.0).build();
        assert_eq!(ax.start, Some(0.0));
        assert_eq!(ax.end, Some(100.0));
    }

    #[test]
    fn axis_config_bounds_sets_min_and_max() {
        let ax = AxisConfig::builder().bounds(10.0, 200.0).build();
        assert_eq!(ax.bounds_min, Some(10.0));
        assert_eq!(ax.bounds_max, Some(200.0));
    }

    #[test]
    fn axis_config_label_rotation() {
        let ax = AxisConfig::builder().label_rotation(45.0).build();
        assert_eq!(ax.label_rotation, Some(45.0));
    }

    #[test]
    fn axis_config_tick_format() {
        let ax = AxisConfig::builder().tick_format("$0,0").build();
        assert_eq!(ax.tick_format.as_deref(), Some("$0,0"));
    }

    #[test]
    fn axis_config_show_grid_false() {
        let ax = AxisConfig::builder().show_grid(false).build();
        assert!(!ax.show_grid);
    }

    #[test]
    fn axis_config_full_chain() {
        let ax = AxisConfig::builder()
            .range(0.0, 500.0)
            .bounds(0.0, 600.0)
            .label_rotation(30.0)
            .tick_format("0.0%")
            .show_grid(false)
            .build();
        assert_eq!(ax.start, Some(0.0));
        assert_eq!(ax.end, Some(500.0));
        assert_eq!(ax.bounds_min, Some(0.0));
        assert_eq!(ax.bounds_max, Some(600.0));
        assert_eq!(ax.label_rotation, Some(30.0));
        assert_eq!(ax.tick_format.as_deref(), Some("0.0%"));
        assert!(!ax.show_grid);
    }
}
