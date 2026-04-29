//! Per-test sensor data generator.
//!
//! Each test (one of six anomaly types) is an independent run with its own
//! datetime baseline.  All 4 sensors are recorded hourly for the duration of
//! the test.  Affected sensors get a half-sine anomaly envelope injected into
//! a defined window.
//!
//! Output is two flavors of DataFrame:
//! 1. **Per-test long form**: one row per (timestamp, sensor) for line / hist plots.
//! 2. **Per-sensor summary**: one row per (test) with min/max/mean/std/affected.

use polars::prelude::*;

pub const SENSORS: [&str; 4] = ["Temperature", "Pressure", "Vibration", "Flow Rate"];
pub const SENSOR_KEYS: [&str; 4] = ["temperature", "pressure", "vibration", "flow_rate"];
pub const UNITS: [&str; 4] = ["°C", "PSI", "mm/s", "L/min"];

const BASELINES: [f64; 4] = [74.0, 118.0, 2.4, 448.0];
const SCALES:    [f64; 4] = [7.0,  14.0,  0.8, 50.0 ];

pub struct TestSpec {
    pub key:           &'static str,   // url-safe slug
    pub label:         &'static str,
    pub color:         &'static str,   // CSS color string for sidebar dot + accents
    pub hours:         i64,
    pub anomaly_start: i64,
    pub anomaly_end:   i64,
    pub affected:      &'static [usize], // sensor indices
    pub severity:      f64,
}

pub const TESTS: [TestSpec; 6] = [
    TestSpec { key: "overtemp",       label: "Overtemp",       color: "oklch(60% 0.18 28)",  hours: 72, anomaly_start: 28, anomaly_end: 38, affected: &[0],          severity:  2.9 },
    TestSpec { key: "high-pressure",  label: "High Pressure",  color: "oklch(70% 0.16 70)",  hours: 60, anomaly_start: 22, anomaly_end: 30, affected: &[1, 0],       severity:  2.6 },
    TestSpec { key: "high-vibration", label: "High Vibration", color: "oklch(58% 0.18 305)", hours: 48, anomaly_start: 18, anomaly_end: 25, affected: &[2],          severity:  3.1 },
    TestSpec { key: "low-flow",       label: "Low Flow",       color: "oklch(64% 0.13 175)", hours: 64, anomaly_start: 30, anomaly_end: 40, affected: &[3, 1],       severity: -2.7 },
    TestSpec { key: "sensor-fault",   label: "Sensor Fault",   color: "oklch(60% 0.14 245)", hours: 36, anomaly_start: 14, anomaly_end: 18, affected: &[0, 1],       severity:  4.2 },
    TestSpec { key: "cascade",        label: "Cascade",        color: "oklch(60% 0.18 350)", hours: 80, anomaly_start: 35, anomaly_end: 50, affected: &[0,1,2,3],    severity:  2.8 },
];

// Deterministic xorshift PRNG so output is reproducible.
struct Xor(u64);
impl Xor {
    fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.0 = x;
        (x as u32 as f64) / (u32::MAX as f64)
    }
    fn normal(&mut self) -> f64 {
        let u = self.next().max(1e-10);
        let v = self.next();
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

const HOUR_MS: i64 = 3_600_000;
/// Test t=0 timestamp: 2024-01-01T00:00:00Z. Per-test datasets share the same
/// reference instant — they are independent runs, not segments of one timeline.
pub const T0_MS: i64 = 1_704_067_200_000;

/// Long-form per-test dataframe: timestamp_ms + one column per sensor.
pub fn build_test_df(spec: &TestSpec) -> DataFrame {
    let mut rng = Xor(0xdead_beef_u64.wrapping_add(spec.key.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64))));
    let n = spec.hours as usize;

    let mut ts = Vec::with_capacity(n);
    for i in 0..(spec.hours) {
        ts.push(T0_MS + i * HOUR_MS);
    }

    let mut cols = vec![Vec::with_capacity(n); 4];
    for i in 0..(spec.hours) {
        for s in 0..4 {
            let base  = BASELINES[s];
            let scale = SCALES[s];
            let daily = ((i as f64) / 24.0 * std::f64::consts::TAU).sin() * scale * 0.12;
            let noise = rng.normal() * scale * 0.18;
            let drift = i as f64 * scale * 0.0005;
            let mut v = base + daily + noise + drift;

            if i >= spec.anomaly_start && i < spec.anomaly_end && spec.affected.contains(&s) {
                let phase = (i - spec.anomaly_start) as f64 / (spec.anomaly_end - spec.anomaly_start) as f64;
                let envelope = (phase * std::f64::consts::PI).sin();
                let sign = if spec.severity >= 0.0 { 1.0 } else { -1.0 };
                v += sign * spec.severity.abs() * envelope * scale * 0.65 + rng.normal() * scale * 0.15;
            }
            cols[s].push((v * 100.0).round() / 100.0);
        }
    }

    df![
        "timestamp_ms" => ts,
        SENSOR_KEYS[0] => cols[0].clone(),
        SENSOR_KEYS[1] => cols[1].clone(),
        SENSOR_KEYS[2] => cols[2].clone(),
        SENSOR_KEYS[3] => cols[3].clone(),
    ].unwrap()
}

/// Stats for a single sensor over a single test.
pub struct TestStats {
    pub min:    f64,
    pub max:    f64,
    pub mean:   f64,
    pub stddev: f64,
}

pub fn stats_of(values: &[f64]) -> TestStats {
    let n = values.len() as f64;
    let sum: f64 = values.iter().sum();
    let mean = sum / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    TestStats {
        min: values.iter().copied().fold(f64::INFINITY, f64::min),
        max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        mean,
        stddev: var.sqrt(),
    }
}

/// Per-sensor cross-test rollup DF: one row per test.
/// Columns: test, color, mean, stddev, min, max, peak_delta, affected (0/1).
pub fn build_summary_df(sensor_idx: usize, test_dfs: &[(String, DataFrame)]) -> DataFrame {
    let baseline = BASELINES[sensor_idx];
    let mut tests   = Vec::new();
    let mut colors  = Vec::new();
    let mut means   = Vec::new();
    let mut stds    = Vec::new();
    let mut mins    = Vec::new();
    let mut maxs    = Vec::new();
    let mut deltas  = Vec::new();
    let mut affs    = Vec::new();

    for spec in TESTS.iter() {
        let df = &test_dfs.iter().find(|(k, _)| k == spec.key).expect("test df present").1;
        let series = df.column(SENSOR_KEYS[sensor_idx]).unwrap().f64().unwrap();
        let vals: Vec<f64> = series.into_no_null_iter().collect();
        let s = stats_of(&vals);

        let affected = spec.affected.contains(&sensor_idx);
        let peak_delta = if affected {
            if spec.severity >= 0.0 { s.max - baseline } else { baseline - s.min }
        } else { 0.0 };

        tests.push(format!(
            r#"<a href="test-{key}.html">{label}</a>"#,
            key = spec.key,
            label = spec.label,
        ));
        colors.push(spec.color.to_string());
        means.push((s.mean * 100.0).round() / 100.0);
        stds.push((s.stddev * 100.0).round() / 100.0);
        mins.push((s.min * 100.0).round() / 100.0);
        maxs.push((s.max * 100.0).round() / 100.0);
        deltas.push((peak_delta * 100.0).round() / 100.0);
        affs.push(if affected { 1i64 } else { 0i64 });
    }

    df![
        "test"       => tests,
        "color"      => colors,
        "mean"       => means,
        "stddev"     => stds,
        "min"        => mins,
        "max"        => maxs,
        "peak_delta" => deltas,
        "affected"   => affs,
    ].unwrap()
}

/// Long-form per-sensor distribution dataframe: one row per (test, value)
/// for box plots showing reading distribution per test.
pub fn build_distribution_df(sensor_idx: usize, test_dfs: &[(String, DataFrame)]) -> DataFrame {
    let mut tests:  Vec<String> = Vec::new();
    let mut values: Vec<f64>    = Vec::new();
    for spec in TESTS.iter() {
        let df = &test_dfs.iter().find(|(k, _)| k == spec.key).expect("test df present").1;
        let series = df.column(SENSOR_KEYS[sensor_idx]).unwrap().f64().unwrap();
        for v in series.into_no_null_iter() {
            tests.push(spec.label.to_string());
            values.push(v);
        }
    }
    df!["test" => tests, "value" => values].unwrap()
}
