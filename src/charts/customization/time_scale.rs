/// Time unit scale used for formatting datetime values on axes and tooltips.
///
/// Each variant maps to a [`DatetimeTickFormatter`](https://docs.bokeh.org/en/latest/docs/reference/models/formatters.html#bokeh.models.formatters.DatetimeTickFormatter)
/// format string and a corresponding tooltip strftime pattern.
///
/// # Format strings produced
///
/// | Variant | Axis tick format | Tooltip format |
/// |---|---|---|
/// | `Milliseconds` | `%H:%M:%S.%3N` | `%H:%M:%S.%3N` |
/// | `Seconds` | `%H:%M:%S` | `%H:%M:%S` |
/// | `Minutes` | `%H:%M` | `%H:%M` |
/// | `Hours` | `%m/%d %H:%M` | `%m/%d %H:%M` |
/// | `Days` | `%Y-%m-%d` | `%Y-%m-%d` |
/// | `Months` | `%b %Y` | `%b %Y` |
/// | `Years` | `%Y` | `%Y` |
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let x = AxisConfig::builder()
///     .time_scale(TimeScale::Days)
///     .build();
/// ```
#[derive(Clone)]
pub enum TimeScale {
    /// Sub-second resolution: `%H:%M:%S.%3N`
    Milliseconds,
    /// Second resolution: `%H:%M:%S`
    Seconds,
    /// Minute resolution: `%H:%M`
    Minutes,
    /// Hour resolution: `%m/%d %H:%M`
    Hours,
    /// Day resolution: `%Y-%m-%d`
    Days,
    /// Month resolution: `%b %Y`
    Months,
    /// Year resolution: `%Y`
    Years,
}

impl TimeScale {
    /// Return the strftime format string used for this scale.
    #[must_use]
    pub fn format_str(&self) -> &'static str {
        match self {
            TimeScale::Milliseconds => "%H:%M:%S.%3N",
            TimeScale::Seconds => "%H:%M:%S",
            TimeScale::Minutes => "%H:%M",
            TimeScale::Hours => "%m/%d %H:%M",
            TimeScale::Days => "%Y-%m-%d",
            TimeScale::Months => "%b %Y",
            TimeScale::Years => "%Y",
        }
    }

    /// Return the string identifier passed to Python.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeScale::Milliseconds => "milliseconds",
            TimeScale::Seconds => "seconds",
            TimeScale::Minutes => "minutes",
            TimeScale::Hours => "hours",
            TimeScale::Days => "days",
            TimeScale::Months => "months",
            TimeScale::Years => "years",
        }
    }
}
