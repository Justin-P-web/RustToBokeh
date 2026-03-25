use super::time_scale::TimeScale;

/// Format applied to a single tooltip field.
///
/// Used inside [`TooltipSpec`] to control how each hover value is displayed.
pub enum TooltipFormat {
    /// Plain text — renders the value as-is.
    Text,
    /// Fixed-point number.  Decimal places default to `2` when `None`.
    Number(Option<u8>),
    /// Percentage.  The raw value is shown with a `%` suffix.
    /// Decimal places default to `1` when `None`.
    Percent(Option<u8>),
    /// Currency — prefixed with `$` and formatted with thousand separators.
    Currency,
    /// Datetime value stored as milliseconds since the Unix epoch.
    /// The [`TimeScale`] controls the strftime display format.
    DateTime(TimeScale),
}

/// A single row in a chart tooltip.
pub struct TooltipField {
    /// Column name in the data source.
    pub column: String,
    /// Human-readable label shown before the value.
    pub label: String,
    /// How to format the column value.
    pub format: TooltipFormat,
}

/// Custom tooltip definition for a chart.
///
/// When provided, replaces the default Bokeh `HoverTool` tooltip with the
/// specified fields in the order they were added.  If omitted the renderer
/// falls back to a sensible default based on the chart's column names.
///
/// Build with [`TooltipSpec::builder`].
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let tt = TooltipSpec::builder()
///     .field("region",  "Region",  TooltipFormat::Text)
///     .field("revenue", "Revenue", TooltipFormat::Currency)
///     .field("growth",  "Growth",  TooltipFormat::Percent(Some(1)))
///     .build();
/// ```
pub struct TooltipSpec {
    /// Ordered list of fields to show in the tooltip.
    pub fields: Vec<TooltipField>,
}

/// Builder for [`TooltipSpec`].
///
/// Call [`field`](TooltipSpecBuilder::field) once per tooltip row, then
/// [`build`](TooltipSpecBuilder::build).
pub struct TooltipSpecBuilder {
    fields: Vec<TooltipField>,
}

impl TooltipSpec {
    /// Create a new builder for a tooltip specification.
    #[must_use]
    pub fn builder() -> TooltipSpecBuilder {
        TooltipSpecBuilder { fields: Vec::new() }
    }
}

impl TooltipSpecBuilder {
    /// Add a field row to the tooltip.
    ///
    /// Fields appear in the order they are added.
    #[must_use]
    pub fn field(mut self, column: &str, label: &str, format: TooltipFormat) -> Self {
        self.fields.push(TooltipField {
            column: column.into(),
            label: label.into(),
            format,
        });
        self
    }

    /// Consume the builder and produce a [`TooltipSpec`].
    #[must_use]
    pub fn build(self) -> TooltipSpec {
        TooltipSpec { fields: self.fields }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TooltipSpec / TooltipSpecBuilder ──────────────────────────────────────

    #[test]
    fn tooltip_spec_builder_empty() {
        let tt = TooltipSpec::builder().build();
        assert!(tt.fields.is_empty());
    }

    #[test]
    fn tooltip_spec_builder_single_field() {
        let tt = TooltipSpec::builder()
            .field("col", "Label", TooltipFormat::Text)
            .build();
        assert_eq!(tt.fields.len(), 1);
        assert_eq!(tt.fields[0].column, "col");
        assert_eq!(tt.fields[0].label, "Label");
        assert!(matches!(tt.fields[0].format, TooltipFormat::Text));
    }

    #[test]
    fn tooltip_spec_builder_field_ordering_preserved() {
        let tt = TooltipSpec::builder()
            .field("a", "A", TooltipFormat::Text)
            .field("b", "B", TooltipFormat::Number(None))
            .field("c", "C", TooltipFormat::Currency)
            .build();
        assert_eq!(tt.fields.len(), 3);
        assert_eq!(tt.fields[0].column, "a");
        assert_eq!(tt.fields[1].column, "b");
        assert_eq!(tt.fields[2].column, "c");
    }

    #[test]
    fn tooltip_format_number_with_decimals() {
        let tt = TooltipSpec::builder()
            .field("v", "V", TooltipFormat::Number(Some(3)))
            .build();
        match tt.fields[0].format {
            TooltipFormat::Number(Some(d)) => assert_eq!(d, 3),
            _ => panic!("expected Number(Some(3))"),
        }
    }

    #[test]
    fn tooltip_format_number_no_decimals() {
        let tt = TooltipSpec::builder()
            .field("v", "V", TooltipFormat::Number(None))
            .build();
        assert!(matches!(tt.fields[0].format, TooltipFormat::Number(None)));
    }

    #[test]
    fn tooltip_format_percent_with_decimals() {
        let tt = TooltipSpec::builder()
            .field("v", "V", TooltipFormat::Percent(Some(2)))
            .build();
        match tt.fields[0].format {
            TooltipFormat::Percent(Some(d)) => assert_eq!(d, 2),
            _ => panic!("expected Percent(Some(2))"),
        }
    }

    #[test]
    fn tooltip_format_currency() {
        let tt = TooltipSpec::builder()
            .field("v", "V", TooltipFormat::Currency)
            .build();
        assert!(matches!(tt.fields[0].format, TooltipFormat::Currency));
    }
}
