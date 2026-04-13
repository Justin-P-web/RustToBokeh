//! Chart builders for native Bokeh rendering.

pub mod box_plot;
pub mod density;
pub mod grouped_bar;
pub mod hbar;
pub mod histogram;
pub mod line;
pub mod pie;
pub mod scatter;

use crate::charts::{ChartConfig, ChartSpec, TooltipField, TooltipFormat, TooltipSpec};
use crate::error::ChartError;
use polars::prelude::DataFrame;
use std::collections::HashMap;

use super::document::BokehDocument;
use super::figure::build_hover_tool;
use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

/// Context passed to every chart builder.
pub struct ChartContext<'a> {
    pub id_gen: &'a mut IdGen,
    pub doc: &'a mut BokehDocument,
    pub frames: &'a HashMap<String, DataFrame>,
    /// CDSView filter reference for filtered charts (None if not filtered).
    pub filter_ref: Option<BokehValue>,
    /// Shared Range1d ID for RangeTool synchronisation (None if not used).
    pub range_tool_x_range_id: Option<String>,
}

/// Build a chart figure `BokehObject` without adding it to any document.
///
/// This lower-level function lets callers inspect the figure (e.g. to extract
/// the CDS ID for filter wiring) before deciding how to embed it.
pub fn build_chart_obj(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    frames: &HashMap<String, DataFrame>,
    filter_ref: Option<BokehValue>,
    range_tool_x_range_id: Option<&str>,
) -> Result<BokehObject, ChartError> {
    let df = frames.get(&spec.source_key).ok_or_else(|| {
        ChartError::NativeRender(format!(
            "source_key '{}' not registered",
            spec.source_key
        ))
    })?;

    match &spec.config {
        ChartConfig::HBar(c) => hbar::build_hbar(id_gen, spec, c, df, filter_ref),
        ChartConfig::Scatter(c) => scatter::build_scatter(
            id_gen, spec, c, df, filter_ref, range_tool_x_range_id,
        ),
        ChartConfig::Histogram(c) => histogram::build_histogram(id_gen, spec, c, df, filter_ref),
        ChartConfig::Line(c) => line::build_line(
            id_gen, spec, c, df, filter_ref, range_tool_x_range_id,
        ),
        ChartConfig::Pie(c) => pie::build_pie(id_gen, spec, c, df),
        ChartConfig::GroupedBar(c) => grouped_bar::build_grouped_bar(id_gen, spec, c, df, filter_ref),
        ChartConfig::BoxPlot(c) => {
            let outlier_df = c.outlier_source_key.as_ref().and_then(|k| frames.get(k));
            box_plot::build_box_plot(id_gen, spec, c, df, outlier_df, filter_ref)
        }
        ChartConfig::Density(c) => density::build_density(id_gen, spec, c, df, filter_ref),
    }
}

/// Build a chart figure for the given `ChartSpec` and add it to the document.
///
/// Returns the HTML div UUID for embedding.
pub fn build_chart(
    ctx: &mut ChartContext<'_>,
    spec: &ChartSpec,
) -> Result<String, ChartError> {
    let fig = build_chart_obj(
        ctx.id_gen,
        spec,
        ctx.frames,
        ctx.filter_ref.clone(),
        ctx.range_tool_x_range_id.as_deref(),
    )?;
    Ok(ctx.doc.add_root(fig))
}

// ── Tooltip helpers ─────────────────────────────────────────────────────────

/// Convert a `TooltipSpec` to `(tooltips, formatters)` for `build_hover_tool`.
pub fn tooltip_arrays(
    spec: &TooltipSpec,
) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let mut tooltips = Vec::new();
    let mut formatters = Vec::new();

    for field in &spec.fields {
        let (fmt_str, fmt_type) = format_tooltip_field(field);
        tooltips.push((field.label.clone(), fmt_str.clone()));
        if let Some(ft) = fmt_type {
            formatters.push((format!("@{{{}}}", field.column), ft));
        }
    }
    (tooltips, formatters)
}

fn format_tooltip_field(f: &TooltipField) -> (String, Option<String>) {
    let col = &f.column;
    match &f.format {
        TooltipFormat::Text => (format!("@{{{col}}}"), None),
        TooltipFormat::Number(dec) => {
            let d = dec.unwrap_or(2) as usize;
            let zeros = "0".repeat(d);
            (format!("@{{{col}}}{{0.{zeros}}}"), None)
        }
        TooltipFormat::Percent(dec) => {
            let d = dec.unwrap_or(1) as usize;
            let zeros = "0".repeat(d);
            (format!("@{{{col}}}{{0.{zeros}%}}"), None)
        }
        TooltipFormat::Currency => (format!("@{{{col}}}{{$0,0}}"), None),
        TooltipFormat::DateTime(scale) => {
            let fmt = time_scale_strftime(scale);
            (format!("@{{{col}}}{{{fmt}}}"), Some("datetime".to_string()))
        }
    }
}

fn time_scale_strftime(scale: &crate::charts::TimeScale) -> &'static str {
    use crate::charts::TimeScale;
    match scale {
        TimeScale::Milliseconds => "%H:%M:%S.%3N",
        TimeScale::Seconds      => "%H:%M:%S",
        TimeScale::Minutes      => "%H:%M",
        TimeScale::Hours        => "%m/%d %H:%M",
        TimeScale::Days         => "%Y-%m-%d",
        TimeScale::Months       => "%b %Y",
        TimeScale::Years        => "%Y",
    }
}

/// Build a default hover tool from column names.
pub fn default_hover_tool(id_gen: &mut IdGen, cols: &[&str]) -> BokehObject {
    let tips: Vec<(&str, String)> = cols
        .iter()
        .map(|c| (*c, format!("@{{{c}}}")))
        .collect();
    let tip_refs: Vec<(&str, &str)> = tips.iter().map(|(l, v)| (*l, v.as_str())).collect();
    build_hover_tool(id_gen, &tip_refs, &[])
}

/// Build a hover tool from a `TooltipSpec` or fall back to default column names.
pub fn make_hover_tool(
    id_gen: &mut IdGen,
    tt: Option<&TooltipSpec>,
    default_cols: &[&str],
) -> BokehObject {
    if let Some(spec) = tt {
        let (tooltips, formatters) = tooltip_arrays(spec);
        let t_refs: Vec<(&str, &str)> = tooltips.iter().map(|(l, v)| (l.as_str(), v.as_str())).collect();
        let f_refs: Vec<(&str, &str)> = formatters.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        build_hover_tool(id_gen, &t_refs, &f_refs)
    } else {
        default_hover_tool(id_gen, default_cols)
    }
}

/// Add an axis label to the figure's x and y axes.
pub fn set_axis_labels(fig: &mut BokehObject, x_label: &str, y_label: &str) {
    // Find `below` (x-axis) and `left` (y-axis), add axis_label attribute
    for (key, val) in &mut fig.attributes {
        if key == "below" {
            if let BokehValue::Array(axes) = val {
                for ax in axes {
                    if let BokehValue::Object(obj) = ax {
                        if !x_label.is_empty() {
                            obj.attributes.push(("axis_label".to_string(), BokehValue::Str(x_label.to_string())));
                        }
                    }
                }
            }
        }
        if key == "left" {
            if let BokehValue::Array(axes) = val {
                for ax in axes {
                    if let BokehValue::Object(obj) = ax {
                        if !y_label.is_empty() {
                            obj.attributes.push(("axis_label".to_string(), BokehValue::Str(y_label.to_string())));
                        }
                    }
                }
            }
        }
    }
}

/// Add one or more glyph renderers to a Figure's `renderers` list.
pub fn add_renderers(fig: &mut BokehObject, renderers: Vec<BokehObject>) {
    for (key, val) in &mut fig.attributes {
        if key == "renderers" {
            if let BokehValue::Array(arr) = val {
                for r in renderers {
                    arr.push(r.into_value());
                }
                return;
            }
        }
    }
}

/// Add a Legend to the Figure's `center` list.
pub fn add_legend(fig: &mut BokehObject, legend: BokehObject) {
    for (key, val) in &mut fig.attributes {
        if key == "center" {
            if let BokehValue::Array(arr) = val {
                arr.push(legend.into_value());
                return;
            }
        }
    }
}
