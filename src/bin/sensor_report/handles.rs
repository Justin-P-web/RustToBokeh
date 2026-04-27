use polars::prelude::DataFrame;
use rust_to_bokeh::prelude::*;

use crate::data::{self, TestSpec, SENSOR_KEYS, TESTS};

/// All registered DataFrame handles + the per-test owned DataFrames retained
/// for stat extraction in page builders (we read raw values out of them when
/// building StatGrid headers).
pub struct Handles {
    pub test_dfs:        Vec<(String, DataFrame)>, // (test_key, df)
    pub test_handles:    Vec<DfHandle>,            // parallel to TESTS
    pub summary_handles: Vec<DfHandle>,            // parallel to SENSORS
    pub distribution_box_handles: Vec<DfHandle>,   // parallel to SENSORS — box plot stats
}

pub fn register(dash: &mut Dashboard) -> Result<Handles, ChartError> {
    // 1. Per-test long-form DFs.
    let mut test_dfs:     Vec<(String, DataFrame)> = Vec::new();
    let mut test_handles: Vec<DfHandle>            = Vec::new();
    for spec in TESTS.iter() {
        let mut df = data::build_test_df(spec);
        let key = format!("test_{}", spec.key.replace('-', "_"));
        let h = dash.add_df(&key, &mut df)?;
        test_dfs.push((spec.key.to_string(), df));
        test_handles.push(h);
    }

    // 2. Per-sensor cross-test summary DFs.
    let mut summary_handles = Vec::new();
    for s_idx in 0..SENSOR_KEYS.len() {
        let mut df = data::build_summary_df(s_idx, &test_dfs);
        let key = format!("summary_{}", SENSOR_KEYS[s_idx]);
        let h = dash.add_df(&key, &mut df)?;
        summary_handles.push(h);
    }

    // 3. Per-sensor distribution box-plot stats (computed via stats helper).
    let mut distribution_box_handles = Vec::new();
    for s_idx in 0..SENSOR_KEYS.len() {
        let dist_df = data::build_distribution_df(s_idx, &test_dfs);
        let mut stats_df = compute_box_stats(&dist_df, "test", "value")?;
        let key = format!("dist_box_{}", SENSOR_KEYS[s_idx]);
        let h = dash.add_df(&key, &mut stats_df)?;
        distribution_box_handles.push(h);
    }

    Ok(Handles { test_dfs, test_handles, summary_handles, distribution_box_handles })
}

pub fn test_spec(key: &str) -> &'static TestSpec {
    TESTS.iter().find(|t| t.key == key).expect("known test key")
}
