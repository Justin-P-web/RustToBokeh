//! Pre-render validation of a [`Dashboard`](crate::Dashboard) build.
//!
//! Runs before any chart is produced so that failures surface as a single
//! [`ChartError::Validation`] with a clear message instead of appearing as
//! obscure runtime artefacts (Bokeh's "???" for null data, silent no-op
//! filters, empty charts, duplicate page files).
//!
//! ## What is checked
//!
//! **Frames.** Each registered `DataFrame` key must be unique and its Arrow
//! IPC bytes must decode cleanly.
//!
//! **Charts.** For every [`ChartSpec`]:
//! - `source_key` resolves to a registered frame
//! - every column referenced by the config (x, y, group, value, quartiles,
//!   histogram edges, etc.) exists in that frame
//! - every referenced column has zero nulls — Bokeh renders a null value as
//!   the literal string `???` in tooltips and axes, which the project treats
//!   as a data-quality failure
//! - chart-type-specific shape checks: histograms require the stat column
//!   for their display mode; box plots require quartiles + whiskers and, if
//!   configured, a matching outlier frame.
//!
//! **Filters.** For every [`FilterSpec`]:
//! - `source_key` + `column` resolve
//! - widget bounds are internally consistent (min < max, step > 0, ...)
//! - the column's dtype matches what the filter widget does to it:
//!   [`FilterConfig::Range`] / [`FilterConfig::Threshold`] /
//!   [`FilterConfig::DateRange`] / [`FilterConfig::RangeTool`] require a
//!   numeric column; [`FilterConfig::Select`] / [`FilterConfig::Group`]
//!   require a string-like (String / Categorical / Enum) column
//! - [`FilterConfig::RangeTool`] has at least one line or scatter chart on
//!   the same page that shares `source_key` — otherwise the navigator
//!   animates nothing.
//!
//! **Tables.** Each column key exists in the referenced frame with zero nulls.
//!
//! **Pages.** Page slugs are unique across the dashboard (duplicate slugs
//! overwrite each other's HTML file). Per-page grid geometry is already
//! enforced by [`PageBuilder::build`](crate::pages::PageBuilder::build).

use std::collections::{HashMap, HashSet};
use std::io::Cursor;

use polars::io::ipc::IpcReader;
use polars::io::SerReader;
use polars::prelude::{DataFrame, DataType};

use crate::charts::{ChartConfig, ChartSpec, FilterConfig, FilterSpec};
use crate::error::ChartError;
use crate::modules::{PageModule, TableSpec};
use crate::pages::Page;

/// Decoded schema + null counts for one registered frame. Built once per
/// validation pass and reused for every spec that references the frame.
struct FrameInfo {
    cols: HashMap<String, ColInfo>,
}

struct ColInfo {
    dtype: DataType,
    null_count: usize,
}

impl FrameInfo {
    fn from_df(df: &DataFrame) -> Self {
        let mut cols = HashMap::with_capacity(df.width());
        for s in df.columns() {
            cols.insert(
                s.name().to_string(),
                ColInfo {
                    dtype: s.dtype().clone(),
                    null_count: s.null_count(),
                },
            );
        }
        Self { cols }
    }
}

fn is_numeric(dt: &DataType) -> bool {
    matches!(
        dt,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
    )
}

fn is_stringy(dt: &DataType) -> bool {
    matches!(
        dt,
        DataType::String | DataType::Categorical(_, _) | DataType::Enum(_, _)
    )
}

/// Validate every invariant listed in the module docs.
///
/// Called automatically by [`Dashboard::render`](crate::Dashboard::render) and
/// [`Dashboard::render_native`](crate::Dashboard::render_native) before any
/// HTML is produced.
///
/// # Errors
///
/// Returns [`ChartError::Validation`] with a human-readable message on the
/// first failure detected. Frame IPC decode failures are wrapped in
/// [`ChartError::Serialization`].
pub fn validate_dashboard(
    frame_data: &[(&str, Vec<u8>)],
    pages: &[Page],
) -> Result<(), ChartError> {
    let frames = decode_frames(frame_data)?;
    check_page_slugs_unique(pages)?;

    for page in pages {
        validate_page(page, &frames)?;
    }
    Ok(())
}

fn decode_frames(
    frame_data: &[(&str, Vec<u8>)],
) -> Result<HashMap<String, FrameInfo>, ChartError> {
    let mut out: HashMap<String, FrameInfo> = HashMap::with_capacity(frame_data.len());
    for (key, bytes) in frame_data {
        if out.contains_key(*key) {
            return Err(ChartError::Validation(format!(
                "duplicate frame key '{key}' — every Dashboard::add_df key must be unique"
            )));
        }
        let df = IpcReader::new(Cursor::new(bytes.as_slice())).finish()?;
        out.insert((*key).to_string(), FrameInfo::from_df(&df));
    }
    Ok(out)
}

fn check_page_slugs_unique(pages: &[Page]) -> Result<(), ChartError> {
    let mut seen: HashSet<&str> = HashSet::with_capacity(pages.len());
    for p in pages {
        if !seen.insert(p.slug.as_str()) {
            return Err(ChartError::Validation(format!(
                "duplicate page slug '{}' — each page writes <slug>.html and would overwrite",
                p.slug
            )));
        }
    }
    Ok(())
}

fn validate_page(
    page: &Page,
    frames: &HashMap<String, FrameInfo>,
) -> Result<(), ChartError> {
    // Collect chart source keys so RangeTool can check that at least one
    // line or scatter chart shares its source.
    for module in &page.modules {
        match module {
            PageModule::Chart(spec) => validate_chart(page, spec, frames)?,
            PageModule::Table(spec) => validate_table(page, spec, frames)?,
            PageModule::Paragraph(_) => {}
        }
    }

    for filter in &page.filters {
        validate_filter(page, filter, frames)?;
    }
    Ok(())
}

// ── chart ────────────────────────────────────────────────────────────────────

fn validate_chart(
    page: &Page,
    spec: &ChartSpec,
    frames: &HashMap<String, FrameInfo>,
) -> Result<(), ChartError> {
    let frame = frames.get(&spec.source_key).ok_or_else(|| {
        ChartError::Validation(format!(
            "chart '{}' on page '{}' references unknown source_key '{}'",
            spec.title, page.slug, spec.source_key
        ))
    })?;

    let loc = ChartLoc {
        page: &page.slug,
        title: &spec.title,
        source: &spec.source_key,
    };

    match &spec.config {
        ChartConfig::GroupedBar(c) => {
            require_col(&loc, frame, &c.x_col, "x")?;
            require_col(&loc, frame, &c.group_col, "group")?;
            require_numeric(&loc, frame, &c.value_col, "value")?;
        }
        ChartConfig::Line(c) => {
            require_col(&loc, frame, &c.x_col, "x")?;
            if c.y_cols.is_empty() {
                return Err(loc.err("y_cols is empty — a line chart must plot at least one series"));
            }
            for y in &c.y_cols {
                require_numeric(&loc, frame, y, "y")?;
            }
        }
        ChartConfig::HBar(c) => {
            require_col(&loc, frame, &c.category_col, "category")?;
            require_numeric(&loc, frame, &c.value_col, "value")?;
        }
        ChartConfig::Scatter(c) => {
            require_numeric(&loc, frame, &c.x_col, "x")?;
            require_numeric(&loc, frame, &c.y_col, "y")?;
        }
        ChartConfig::Pie(c) => {
            require_col(&loc, frame, &c.label_col, "label")?;
            require_numeric(&loc, frame, &c.value_col, "value")?;
        }
        ChartConfig::Histogram(c) => {
            require_numeric(&loc, frame, "left", "left edge")?;
            require_numeric(&loc, frame, "right", "right edge")?;
            let stat = c
                .display
                .as_ref()
                .map(|d| d.as_str())
                .unwrap_or("count");
            require_numeric(&loc, frame, stat, "histogram stat")?;
        }
        ChartConfig::BoxPlot(c) => {
            require_col(&loc, frame, &c.category_col, "category")?;
            require_numeric(&loc, frame, &c.q1_col, "q1")?;
            require_numeric(&loc, frame, &c.q2_col, "q2")?;
            require_numeric(&loc, frame, &c.q3_col, "q3")?;
            require_numeric(&loc, frame, &c.lower_col, "lower")?;
            require_numeric(&loc, frame, &c.upper_col, "upper")?;
            if let Some(outlier_key) = &c.outlier_source_key {
                let outlier_frame = frames.get(outlier_key).ok_or_else(|| {
                    ChartError::Validation(format!(
                        "box plot '{}' on page '{}' references unknown outlier source_key '{}'",
                        spec.title, page.slug, outlier_key
                    ))
                })?;
                let outlier_col = c.outlier_value_col.as_deref().ok_or_else(|| {
                    loc.err(
                        "outlier_source_key set without outlier_value_col — supply the numeric column",
                    )
                })?;
                require_col_in(
                    outlier_frame,
                    outlier_col,
                    &format!(
                        "outlier value column (box plot '{}' on page '{}', outlier source '{}')",
                        spec.title, page.slug, outlier_key
                    ),
                )?;
                require_numeric_in(
                    outlier_frame,
                    outlier_col,
                    &format!(
                        "outlier value column (box plot '{}' on page '{}', outlier source '{}')",
                        spec.title, page.slug, outlier_key
                    ),
                )?;
                // category column is reused from the primary frame name
                require_col_in(
                    outlier_frame,
                    &c.category_col,
                    &format!(
                        "category column (box plot '{}' on page '{}', outlier source '{}')",
                        spec.title, page.slug, outlier_key
                    ),
                )?;
            }
        }
        ChartConfig::Density(c) => {
            require_col(&loc, frame, &c.category_col, "category")?;
            require_numeric(&loc, frame, &c.value_col, "value")?;
        }
    }
    Ok(())
}

// ── filter ───────────────────────────────────────────────────────────────────

fn validate_filter(
    page: &Page,
    filter: &FilterSpec,
    frames: &HashMap<String, FrameInfo>,
) -> Result<(), ChartError> {
    let frame = frames.get(&filter.source_key).ok_or_else(|| {
        ChartError::Validation(format!(
            "filter '{}' on page '{}' references unknown source_key '{}'",
            filter.label, page.slug, filter.source_key
        ))
    })?;

    let col = frame.cols.get(&filter.column).ok_or_else(|| {
        ChartError::Validation(format!(
            "filter '{}' on page '{}' references column '{}' which does not exist in frame '{}'",
            filter.label, page.slug, filter.column, filter.source_key
        ))
    })?;

    if col.null_count != 0 {
        return Err(ChartError::Validation(format!(
            "filter '{}' on page '{}' targets column '{}' of frame '{}' which has {} null(s) — \
             Bokeh would render them as '???' and skip them on filter change",
            filter.label, page.slug, filter.column, filter.source_key, col.null_count,
        )));
    }

    match &filter.config {
        FilterConfig::Range { min, max, step } => {
            if !(min < max) {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': Range min ({min}) must be strictly less than max ({max})",
                    filter.label, page.slug,
                )));
            }
            if !(*step > 0.0) {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': Range step ({step}) must be strictly positive",
                    filter.label, page.slug,
                )));
            }
            if !is_numeric(&col.dtype) {
                return Err(dtype_err(
                    &filter.label, &page.slug, &filter.column, &col.dtype, "Range", "numeric",
                ));
            }
        }
        FilterConfig::Select { options } | FilterConfig::Group { options } => {
            if options.is_empty() {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': option list is empty",
                    filter.label, page.slug,
                )));
            }
            if !is_stringy(&col.dtype) {
                return Err(dtype_err(
                    &filter.label,
                    &page.slug,
                    &filter.column,
                    &col.dtype,
                    match &filter.config {
                        FilterConfig::Select { .. } => "Select",
                        _ => "Group",
                    },
                    "String / Categorical / Enum",
                ));
            }
        }
        FilterConfig::Threshold { .. } => {
            if !is_numeric(&col.dtype) {
                return Err(dtype_err(
                    &filter.label, &page.slug, &filter.column, &col.dtype, "Threshold", "numeric",
                ));
            }
        }
        FilterConfig::TopN { max_n, .. } => {
            if *max_n == 0 {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': TopN max_n must be at least 1",
                    filter.label, page.slug,
                )));
            }
        }
        FilterConfig::DateRange { min_ms, max_ms, .. } => {
            if !(min_ms < max_ms) {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': DateRange min_ms ({min_ms}) must be strictly less than max_ms ({max_ms})",
                    filter.label, page.slug,
                )));
            }
            if !is_numeric(&col.dtype) {
                return Err(dtype_err(
                    &filter.label, &page.slug, &filter.column, &col.dtype, "DateRange",
                    "numeric (ms since Unix epoch)",
                ));
            }
        }
        FilterConfig::RangeTool { y_column, start, end, .. } => {
            if !(start < end) {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': RangeTool start ({start}) must be strictly less than end ({end})",
                    filter.label, page.slug,
                )));
            }
            if !is_numeric(&col.dtype) {
                return Err(dtype_err(
                    &filter.label, &page.slug, &filter.column, &col.dtype, "RangeTool", "numeric",
                ));
            }
            require_col_in(
                frame,
                y_column,
                &format!(
                    "y_column (RangeTool filter '{}' on page '{}', source '{}')",
                    filter.label, page.slug, filter.source_key,
                ),
            )?;
            require_numeric_in(
                frame,
                y_column,
                &format!(
                    "y_column (RangeTool filter '{}' on page '{}', source '{}')",
                    filter.label, page.slug, filter.source_key,
                ),
            )?;
            // A RangeTool does nothing unless a line or scatter chart on the
            // same page shares source_key.
            let has_consumer = page.modules.iter().any(|m| match m {
                PageModule::Chart(c) => {
                    c.source_key == filter.source_key
                        && matches!(
                            c.config,
                            ChartConfig::Line(_) | ChartConfig::Scatter(_)
                        )
                }
                _ => false,
            });
            if !has_consumer {
                return Err(ChartError::Validation(format!(
                    "filter '{}' on page '{}': RangeTool has no line or scatter chart sharing \
                     source_key '{}' to synchronise — add a .filtered() chart or remove the navigator",
                    filter.label, page.slug, filter.source_key,
                )));
            }
        }
    }
    Ok(())
}

// ── table ────────────────────────────────────────────────────────────────────

fn validate_table(
    page: &Page,
    spec: &TableSpec,
    frames: &HashMap<String, FrameInfo>,
) -> Result<(), ChartError> {
    let frame = frames.get(&spec.source_key).ok_or_else(|| {
        ChartError::Validation(format!(
            "table '{}' on page '{}' references unknown source_key '{}'",
            spec.title, page.slug, spec.source_key
        ))
    })?;
    if spec.columns.is_empty() {
        return Err(ChartError::Validation(format!(
            "table '{}' on page '{}' has no columns",
            spec.title, page.slug,
        )));
    }
    for col in &spec.columns {
        let info = frame.cols.get(&col.key).ok_or_else(|| {
            ChartError::Validation(format!(
                "table '{}' on page '{}' references column '{}' which does not exist in frame '{}'",
                spec.title, page.slug, col.key, spec.source_key
            ))
        })?;
        if info.null_count != 0 {
            return Err(ChartError::Validation(format!(
                "table '{}' on page '{}' references column '{}' of frame '{}' which has {} null(s) — \
                 Bokeh would render them as '???'",
                spec.title, page.slug, col.key, spec.source_key, info.null_count,
            )));
        }
    }
    Ok(())
}

// ── helpers ──────────────────────────────────────────────────────────────────

struct ChartLoc<'a> {
    page: &'a str,
    title: &'a str,
    source: &'a str,
}

impl ChartLoc<'_> {
    fn err(&self, msg: &str) -> ChartError {
        ChartError::Validation(format!(
            "chart '{}' on page '{}' (source '{}'): {}",
            self.title, self.page, self.source, msg
        ))
    }
}

fn require_col(
    loc: &ChartLoc,
    frame: &FrameInfo,
    name: &str,
    role: &str,
) -> Result<(), ChartError> {
    let info = frame.cols.get(name).ok_or_else(|| {
        loc.err(&format!("{role} column '{name}' does not exist in the frame"))
    })?;
    if info.null_count != 0 {
        return Err(loc.err(&format!(
            "{role} column '{name}' has {} null(s) — Bokeh renders nulls as '???'",
            info.null_count
        )));
    }
    Ok(())
}

fn require_numeric(
    loc: &ChartLoc,
    frame: &FrameInfo,
    name: &str,
    role: &str,
) -> Result<(), ChartError> {
    require_col(loc, frame, name, role)?;
    let info = frame.cols.get(name).expect("checked above");
    if !is_numeric(&info.dtype) {
        return Err(loc.err(&format!(
            "{role} column '{name}' has dtype {:?} — expected numeric",
            info.dtype
        )));
    }
    Ok(())
}

fn require_col_in(frame: &FrameInfo, name: &str, label: &str) -> Result<(), ChartError> {
    let info = frame.cols.get(name).ok_or_else(|| {
        ChartError::Validation(format!(
            "{label}: column '{name}' does not exist in the frame"
        ))
    })?;
    if info.null_count != 0 {
        return Err(ChartError::Validation(format!(
            "{label}: column '{name}' has {} null(s) — Bokeh renders nulls as '???'",
            info.null_count
        )));
    }
    Ok(())
}

fn require_numeric_in(frame: &FrameInfo, name: &str, label: &str) -> Result<(), ChartError> {
    require_col_in(frame, name, label)?;
    let info = frame.cols.get(name).expect("checked above");
    if !is_numeric(&info.dtype) {
        return Err(ChartError::Validation(format!(
            "{label}: column '{name}' has dtype {:?} — expected numeric",
            info.dtype
        )));
    }
    Ok(())
}

fn dtype_err(
    label: &str,
    slug: &str,
    col: &str,
    dt: &DataType,
    kind: &str,
    expected: &str,
) -> ChartError {
    ChartError::Validation(format!(
        "filter '{label}' on page '{slug}': {kind} filter targets column '{col}' \
         with dtype {dt:?} — expected {expected}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{
        BoxPlotConfig, ChartSpecBuilder, DensityConfig, FilterSpec, GroupedBarConfig,
        HBarConfig, HistogramConfig, HistogramDisplay, LineConfig, PieConfig, ScatterConfig,
    };
    use crate::handle::DfHandle;
    use crate::modules::{TableColumn, TableSpec};
    use crate::pages::PageBuilder;
    use crate::serialize_df;
    use polars::prelude::*;

    fn ok_frames(dfs: Vec<(&'static str, DataFrame)>) -> Vec<(&'static str, Vec<u8>)> {
        dfs.into_iter()
            .map(|(k, mut df)| (k, serialize_df(&mut df).unwrap()))
            .collect()
    }

    // ── frames ───────────────────────────────────────────────────────────────

    #[test]
    fn passes_minimal_ok_dashboard() {
        let df = df![
            "month" => ["Jan", "Feb"],
            "revenue" => [10.0f64, 20.0],
        ]
        .unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = LineConfig::builder()
            .x("month").y_cols(&["revenue"]).y_label("USD")
            .build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::line("L", &DfHandle::new("data"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap();
    }

    #[test]
    fn duplicate_frame_key_fails() {
        let df = df!["a" => [1i64]].unwrap();
        let frames = ok_frames(vec![("dup", df.clone()), ("dup", df)]);
        let err = validate_dashboard(&frames, &[]).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("duplicate frame key")));
    }

    // ── pages ────────────────────────────────────────────────────────────────

    #[test]
    fn duplicate_page_slug_fails() {
        let df = df!["a" => ["x"]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = HBarConfig::builder().category("a").value("a").x_label("X").build().unwrap();
        let p1 = PageBuilder::new("slug", "P", "P", 1)
            .chart(ChartSpecBuilder::hbar("C", &DfHandle::new("data"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let cfg2 = HBarConfig::builder().category("a").value("a").x_label("X").build().unwrap();
        let p2 = PageBuilder::new("slug", "P2", "P2", 1)
            .chart(ChartSpecBuilder::hbar("C", &DfHandle::new("data"), cfg2).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, &[p1, p2]).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("duplicate page slug")));
    }

    // ── chart: missing column / null / unknown source ────────────────────────

    #[test]
    fn chart_unknown_source_key_fails() {
        let df = df!["x" => [1i64], "y" => [2.0f64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::scatter("S", &DfHandle::new("missing"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("unknown source_key 'missing'")));
    }

    #[test]
    fn chart_missing_column_fails() {
        let df = df!["x" => [1i64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::scatter("S", &DfHandle::new("data"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("'y' does not exist")));
    }

    #[test]
    fn chart_with_null_column_fails() {
        let df = df![
            "x" => ["a", "b"],
            "y" => [Some(1.0f64), None],
        ].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = HBarConfig::builder().category("x").value("y").x_label("V").build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::hbar("C", &DfHandle::new("data"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("null") && m.contains("???")));
    }

    #[test]
    fn chart_non_numeric_value_column_fails() {
        let df = df![
            "x" => ["a", "b"],
            "y" => ["10", "20"],
        ].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = HBarConfig::builder().category("x").value("y").x_label("V").build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::hbar("C", &DfHandle::new("data"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("expected numeric")));
    }

    #[test]
    fn empty_line_y_cols_fails() {
        let df = df!["x" => [1i64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = LineConfig::builder()
            .x("x").y_cols(&[] as &[&str]).y_label("Y")
            .build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::line("L", &DfHandle::new("data"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("y_cols is empty")));
    }

    #[test]
    fn histogram_missing_stat_col_fails() {
        // left + right present but no cdf column, while display = Cdf
        let df = df![
            "left" => [0.0f64, 1.0],
            "right" => [1.0f64, 2.0],
            "count" => [3.0f64, 5.0],
        ].unwrap();
        let frames = ok_frames(vec![("h", df)]);
        let cfg = HistogramConfig::builder()
            .x_label("v").display(HistogramDisplay::Cdf).build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::histogram("H", &DfHandle::new("h"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("'cdf'")));
    }

    #[test]
    fn box_plot_outlier_source_missing_fails() {
        let df = df![
            "category" => ["A"],
            "q1" => [1.0f64], "q2" => [2.0f64], "q3" => [3.0f64],
            "lower" => [0.0f64], "upper" => [4.0f64],
        ].unwrap();
        let frames = ok_frames(vec![("box", df)]);
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Y")
            .outlier_source("outliers").outlier_value_col("v")
            .build().unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(ChartSpecBuilder::box_plot("B", &DfHandle::new("box"), cfg).at(0, 0, 1).build())
            .build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("unknown outlier source_key 'outliers'")));
    }

    // ── filter ───────────────────────────────────────────────────────────────

    #[test]
    fn filter_unknown_source_fails() {
        let df = df!["x" => [1.0f64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::range(&DfHandle::new("missing"), "x", "X", 0.0, 1.0, 0.1);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("unknown source_key 'missing'")));
    }

    #[test]
    fn filter_range_bad_bounds_fails() {
        let df = df!["x" => [1.0f64, 2.0]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::range(&DfHandle::new("data"), "x", "X", 5.0, 1.0, 0.1);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("Range min")));
    }

    #[test]
    fn filter_range_on_string_column_fails() {
        let df = df!["x" => ["a", "b"]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::range(&DfHandle::new("data"), "x", "X", 0.0, 1.0, 0.1);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("expected numeric")));
    }

    #[test]
    fn filter_select_on_numeric_fails() {
        let df = df!["x" => [1.0f64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::select(&DfHandle::new("data"), "x", "X", vec!["a"]);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("String / Categorical / Enum")));
    }

    #[test]
    fn filter_group_empty_options_fails() {
        let df = df!["x" => ["a"]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::group(&DfHandle::new("data"), "x", "X", vec![]);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("option list is empty")));
    }

    #[test]
    fn filter_top_n_zero_fails() {
        let df = df!["x" => [1.0f64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::top_n(&DfHandle::new("data"), "x", "X", 0, true);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("TopN max_n")));
    }

    #[test]
    fn filter_null_column_fails() {
        let df = df![
            "x" => [Some(1.0f64), None],
        ].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::range(&DfHandle::new("data"), "x", "X", 0.0, 5.0, 0.1);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("null")));
    }

    #[test]
    fn range_tool_without_consumer_fails() {
        let df = df![
            "t" => [0.0f64, 1.0],
            "y" => [10.0f64, 20.0],
        ].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let f = FilterSpec::range_tool(&DfHandle::new("data"), "t", "y", "Nav", 0.0, 1.0, None);
        let page = PageBuilder::new("p", "P", "P", 1).filter(f).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("RangeTool has no line or scatter")));
    }

    #[test]
    fn range_tool_with_scatter_consumer_ok() {
        let df = df![
            "t" => [0.0f64, 1.0],
            "y" => [10.0f64, 20.0],
        ].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let cfg = ScatterConfig::builder()
            .x("t").y("y").x_label("T").y_label("Y")
            .build().unwrap();
        let chart = ChartSpecBuilder::scatter("S", &DfHandle::new("data"), cfg).at(0, 0, 1).build();
        let f = FilterSpec::range_tool(&DfHandle::new("data"), "t", "y", "Nav", 0.0, 1.0, None);
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(chart)
            .filter(f)
            .build().unwrap();
        validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap();
    }

    // ── table ────────────────────────────────────────────────────────────────

    #[test]
    fn table_missing_column_fails() {
        let df = df!["a" => [1i64]].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let tbl = TableSpec::new("T", &DfHandle::new("data"))
            .column(TableColumn::text("not_there", "X"))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1).table(tbl).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("'not_there'")));
    }

    #[test]
    fn table_null_column_fails() {
        let df = df![
            "v" => [Some(1i64), None],
        ].unwrap();
        let frames = ok_frames(vec![("data", df)]);
        let tbl = TableSpec::new("T", &DfHandle::new("data"))
            .column(TableColumn::text("v", "V"))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1).table(tbl).build().unwrap();
        let err = validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap_err();
        assert!(matches!(err, ChartError::Validation(ref m) if m.contains("null")));
    }

    // ── coverage for remaining chart types ──────────────────────────────────

    #[test]
    fn grouped_bar_and_pie_and_density_pass() {
        let df_bar = df![
            "month" => ["Jan", "Feb"],
            "cat"   => ["A", "B"],
            "val"   => [1.0f64, 2.0],
        ].unwrap();
        let df_pie = df![
            "label" => ["x", "y"],
            "value" => [3.0f64, 4.0],
        ].unwrap();
        let df_den = df![
            "dept"     => ["Eng", "Eng"],
            "salary_k" => [100.0f64, 110.0],
        ].unwrap();
        let frames = ok_frames(vec![("bar", df_bar), ("pie", df_pie), ("den", df_den)]);

        let bar_cfg = GroupedBarConfig::builder()
            .x("month").group("cat").value("val").y_label("Y").build().unwrap();
        let pie_cfg = PieConfig::builder().label("label").value("value").build().unwrap();
        let den_cfg = DensityConfig::builder()
            .category("dept").value("salary_k").y_label("Y").build().unwrap();

        let page = PageBuilder::new("p", "P", "P", 3)
            .chart(ChartSpecBuilder::bar("B", &DfHandle::new("bar"), bar_cfg).at(0, 0, 1).build())
            .chart(ChartSpecBuilder::pie("P", &DfHandle::new("pie"), pie_cfg).at(0, 1, 1).build())
            .chart(ChartSpecBuilder::density("D", &DfHandle::new("den"), den_cfg).at(0, 2, 1).build())
            .build().unwrap();

        validate_dashboard(&frames, std::slice::from_ref(&page)).unwrap();
    }
}
