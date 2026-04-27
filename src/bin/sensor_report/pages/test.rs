//! Per-test page: time selector at top + general info + plots for that test.

use rust_to_bokeh::prelude::*;

use crate::data::{SENSORS, SENSOR_KEYS, T0_MS, UNITS};
use crate::handles::{test_spec, Handles};

type C    = ChartSpecBuilder;
type Line = LineConfig;

const HOUR_MS: i64 = 3_600_000;

pub fn build(test_idx: usize, h: &Handles) -> Result<Page, ChartError> {
    let spec    = &crate::data::TESTS[test_idx];
    let _ = test_spec; // re-export sanity
    let test_h  = &h.test_handles[test_idx];

    let (_, df) = &h.test_dfs[test_idx];

    // Compute header tile values per sensor for this test.
    let mut affected_labels: Vec<&str> = spec.affected.iter().map(|&i| SENSORS[i]).collect();
    affected_labels.sort();
    let affected_str = affected_labels.join(", ");

    // Peak deviation across affected sensors (largest |Δ| from baseline).
    let mut peak_delta: f64 = 0.0;
    let mut peak_unit = "";
    for &s_idx in spec.affected.iter() {
        let series = df.column(SENSOR_KEYS[s_idx]).unwrap().f64().unwrap();
        let vals: Vec<f64> = series.into_no_null_iter().collect();
        let stats = crate::data::stats_of(&vals);
        let baseline = match s_idx { 0 => 74.0, 1 => 118.0, 2 => 2.4, _ => 448.0 };
        let d = if spec.severity >= 0.0 { stats.max - baseline } else { baseline - stats.min };
        if d.abs() > peak_delta.abs() {
            peak_delta = d;
            peak_unit  = UNITS[s_idx];
        }
    }

    let n_rows = df.height();
    let slug   = format!("test-{}", spec.key);
    let title  = format!("{} — Test Window", spec.label);
    let nav    = spec.label.to_string();

    // RangeTool overview filters / zooms the line chart's x axis.
    let test_end_ms = T0_MS + spec.hours * HOUR_MS;

    let line_cfg = Line::builder()
        .x("timestamp_ms")
        .y_cols(&["temperature", "pressure", "vibration", "flow_rate"])
        .y_label("Sensor reading")
        .x_axis(AxisConfig::builder().time_scale(TimeScale::Hours).build())
        .tooltips(
            TooltipSpec::builder()
                .field("timestamp_ms", "Time", TooltipFormat::DateTime(TimeScale::Hours))
                .field("temperature",  "Temp (°C)",      TooltipFormat::Number(Some(1)))
                .field("pressure",     "Pressure (PSI)", TooltipFormat::Number(Some(1)))
                .field("vibration",    "Vib (mm/s)",     TooltipFormat::Number(Some(2)))
                .field("flow_rate",    "Flow (L/min)",   TooltipFormat::Number(Some(1)))
                .build(),
        )
        .build()?;

    // Histogram: distribution of the most-affected sensor's readings.
    let primary_sensor_idx = *spec.affected.first().unwrap_or(&0);
    let hist_df_key = format!("test_{}_hist_{}", spec.key.replace('-', "_"), SENSOR_KEYS[primary_sensor_idx]);
    let _ = hist_df_key; // currently unused — placeholder for future per-test hist DF.

    PageBuilder::new(&slug, &title, &nav, 2)
        .category("Tests")
        .dot_color(spec.color)
        .stat_grid(
            StatGridSpec::new()
                .item(StatItem::new("TEST", spec.label))
                .item(StatItem::new("DURATION", &spec.hours.to_string()).suffix("h"))
                .item(StatItem::new("SAMPLES", &n_rows.to_string()))
                .item(StatItem::new("ANOMALY",
                    &format!("{}–{}h", spec.anomaly_start, spec.anomaly_end)))
                .item(StatItem::new("AFFECTED", if affected_str.is_empty() { "—" } else { &affected_str }))
                .item(StatItem::new("PEAK Δ", &format!("{:+.2}", peak_delta)).suffix(peak_unit))
                .at(0, 0, 2)
                .build(),
        )
        .chart(
            C::line(
                "All Sensors — Full Test Window",
                test_h,
                line_cfg,
            )
            .at(1, 0, 2)
            .build(),
        )
        .filter(FilterSpec::range_tool(
            test_h,
            "timestamp_ms",
            "temperature",
            "Time selector — drag the shaded window to zoom",
            T0_MS as f64,
            test_end_ms as f64,
            Some(TimeScale::Hours),
        ))
        .build()
}
