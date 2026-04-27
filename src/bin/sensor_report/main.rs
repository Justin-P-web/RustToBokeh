//! SensorDash sensor-report demo dashboard.
//!
//! Mirrors the Dashboard v2 design from `claude.ai/design`:
//!   * Sidebar grouped into Summary (per-sensor rollup) and Tests (per-event).
//!   * Per-test pages have a time selector at top, general-info stat header,
//!     and plots for the test window.

mod data;
mod handles;
mod pages;

use rust_to_bokeh::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dash = Dashboard::new()
        .title("SensorDash")
        .nav_style(NavStyle::Vertical)
        .output_dir("output_sensor");

    let h = handles::register(&mut dash)?;

    // Summary section — one page per sensor.
    for s_idx in 0..data::SENSORS.len() {
        dash.add_page(pages::summary::build(s_idx, &h)?);
    }
    // Tests section — one page per anomaly type / test.
    for t_idx in 0..data::TESTS.len() {
        dash.add_page(pages::test::build(t_idx, &h)?);
    }

    #[cfg(feature = "python")]
    dash.render()?;
    #[cfg(not(feature = "python"))]
    dash.render_native(BokehResources::Inline)?;

    Ok(())
}
