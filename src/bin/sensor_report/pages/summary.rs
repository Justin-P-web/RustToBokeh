//! Per-sensor cross-test rollup page.
//!
//! One page per sensor.  Shows:
//!   * StatGrid header — count of tests, count where sensor was affected,
//!     baseline reading, max peak Δ across tests.
//!   * Box plot — reading distribution per test (one box per test).
//!   * Stats table — per-test min/max/mean/std/peak Δ + Affected flag.

use rust_to_bokeh::prelude::*;

use crate::data::{SENSORS, SENSOR_KEYS, TESTS, UNITS};
use crate::handles::Handles;

type C = ChartSpecBuilder;
type BP = BoxPlotConfig;
type Tbl = TableSpec;
type TC = TableColumn;

pub fn build(sensor_idx: usize, h: &Handles) -> Result<Page, ChartError> {
    let sensor = SENSORS[sensor_idx];
    let unit   = UNITS[sensor_idx];
    let slug   = format!("summary-{}", SENSOR_KEYS[sensor_idx].replace('_', "-"));
    let nav    = sensor.to_string();

    let summary_h     = &h.summary_handles[sensor_idx];
    let dist_box_h    = &h.distribution_box_handles[sensor_idx];

    // Compute header tile values.
    let summary_df = h.test_dfs.iter()
        .map(|(k, df)| (k.clone(), df))
        .collect::<Vec<_>>();
    let mut affected_tests = 0;
    let mut max_peak_delta: f64 = 0.0;
    for spec in TESTS.iter() {
        let df = summary_df.iter().find(|(k, _)| k == spec.key).unwrap().1;
        let series = df.column(SENSOR_KEYS[sensor_idx]).unwrap().f64().unwrap();
        let vals: Vec<f64> = series.into_no_null_iter().collect();
        let s = crate::data::stats_of(&vals);
        let baseline = match sensor_idx {
            0 => 74.0, 1 => 118.0, 2 => 2.4, _ => 448.0,
        };
        if spec.affected.contains(&sensor_idx) {
            affected_tests += 1;
            let peak = if spec.severity >= 0.0 { s.max - baseline } else { baseline - s.min };
            if peak > max_peak_delta { max_peak_delta = peak; }
        }
    }

    let baseline_txt: String = match sensor_idx {
        0 => "74".into(), 1 => "118".into(), 2 => "2.40".into(), _ => "448".into(),
    };

    PageBuilder::new(&slug, &format!("{sensor} — Cross-Test Summary"), &nav, 2)
        .category("Summary")
        .stat_grid(
            StatGridSpec::new()
                .item(StatItem::new("SENSOR", sensor))
                .item(StatItem::new("UNIT", unit))
                .item(StatItem::new("BASELINE", &baseline_txt).suffix(unit))
                .item(StatItem::new("TESTS", &TESTS.len().to_string()))
                .item(StatItem::new("AFFECTED", &affected_tests.to_string()))
                .item(StatItem::new("MAX PEAK Δ", &format!("{:+.2}", max_peak_delta)).suffix(unit))
                .at(0, 0, 2)
                .build(),
        )
        .chart(
            C::box_plot(
                "Reading Distribution Per Test",
                dist_box_h,
                BP::builder()
                    .category("test")
                    .q1("q1").q2("q2").q3("q3")
                    .lower("lower").upper("upper")
                    .y_label(&format!("{sensor} ({unit})"))
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .table(
            Tbl::new("Per-Test Statistics", summary_h)
                .column(TC::text("test", "Test"))
                .column(TC::number("min",       "Min",       2))
                .column(TC::number("max",       "Max",       2))
                .column(TC::number("mean",      "Mean",      2))
                .column(TC::number("stddev",    "Std Dev",   2))
                .column(TC::number("peak_delta","Peak Δ",    2))
                .column(TC::number("affected",  "Affected",  0))
                .at(2, 0, 2)
                .build(),
        )
        .build()
}
