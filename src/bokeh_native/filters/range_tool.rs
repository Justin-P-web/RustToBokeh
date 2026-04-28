//! RangeTool filter — overview chart with draggable range selector that syncs
//! the shared x-axis Range1d of the page's detail charts.

use polars::prelude::DataFrame;

use crate::charts::{AxisConfig, FilterConfig, FilterSpec};
use crate::error::ChartError;

use super::super::charts::add_renderers;
use super::super::figure::{
    build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind,
};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::source::{build_column_data_source, get_f64_column};
use super::FilterOutput;

pub(super) fn build_range_tool(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    df: &DataFrame,
) -> Result<FilterOutput, ChartError> {
    let (start, end, y_col, time_scale) = match &filter.config {
        FilterConfig::RangeTool { start, end, y_column, time_scale } => {
            (*start, *end, y_column.clone(), time_scale.clone())
        }
        _ => unreachable!(),
    };

    // Shared Range1d for x-axis synchronisation (its ID is used for chart linking)
    let range_id = id_gen.next();

    // BooleanFilter driven by the Range1d (for .filtered() charts)
    let n = df.height();
    let bf_id = id_gen.next();
    let bf = BokehObject::new("BooleanFilter", bf_id.clone())
        .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); n]));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);

    let range_cb_code = format!(
        "const lo = cb_obj.start;\
         const hi = cb_obj.end;\
         const data = source.data['{}'];\
         bf.booleans = data.map(v => v >= lo && v <= hi);\
         source.change.emit();",
        filter.column
    );

    // Inline the BooleanFilter into the FIRST CustomJS args (start_cb). The
    // Range1d widget is a doc root decoded before chart figures, so this is
    // the earliest point at which `bf_id` must be registered for downstream
    // `Ref` sites (end_cb, chart CDSViews) to resolve.
    let start_cb_id = id_gen.next();
    let start_cb = BokehObject::new("CustomJS", start_cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), bf.clone().into_value()),
            ("source".into(), BokehValue::Ref(cds_placeholder_id.clone())),
            ("col".into(), BokehValue::Str(filter.column.clone())),
        ]))
        .attr("code", BokehValue::Str(range_cb_code.clone()));

    let end_cb_id = id_gen.next();
    let end_cb = BokehObject::new("CustomJS", end_cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), BokehValue::ref_of(&bf_id)),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
            ("col".into(), BokehValue::Str(filter.column.clone())),
        ]))
        .attr("code", BokehValue::Str(range_cb_code));

    // Data-derived bounds + small buffer so panning/zoom can't escape data.
    let (bounds_lo, bounds_hi) = compute_bounds(df, &filter.column);
    let clamped_start = start.max(bounds_lo).min(bounds_hi);
    let clamped_end = end.max(bounds_lo).min(bounds_hi);

    let range_widget = BokehObject::new("Range1d", range_id.clone())
        .attr("start", BokehValue::Float(clamped_start))
        .attr("end", BokehValue::Float(clamped_end))
        .attr("bounds", BokehValue::Array(vec![
            BokehValue::Float(bounds_lo),
            BokehValue::Float(bounds_hi),
        ]))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:start".into(), BokehValue::Array(vec![start_cb.into_value()])),
            ("change:end".into(), BokehValue::Array(vec![end_cb.into_value()])),
        ]));

    // Overview figure
    let is_datetime = time_scale.is_some();
    let x_axis_type = if is_datetime { AxisType::Datetime } else { AxisType::Linear };

    let cds = build_column_data_source(id_gen, df);

    // Bound the overview chart's own x and y axes to the data extent (+ buffer)
    // so the user can't pan/zoom the navigator past the data either.
    let overview_x_cfg = AxisConfig::builder()
        .bounds(bounds_lo, bounds_hi)
        .build();
    let (y_bounds_lo, y_bounds_hi) = compute_bounds(df, &y_col);
    let overview_y_cfg = AxisConfig::builder()
        .bounds(y_bounds_lo, y_bounds_hi)
        .build();

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &filter.label,
        130,
        None,
        XRangeKind::DataRange,
        YRangeKind::DataRange,
        AxisBuilder::x(x_axis_type).config(Some(&overview_x_cfg)),
        AxisBuilder::y(AxisType::Linear).config(Some(&overview_y_cfg)),
        None,
    );

    let line_glyph_id = id_gen.next();
    let line_glyph = BokehObject::new("Line", line_glyph_id)
        .attr("x", BokehValue::field(&filter.column))
        .attr("y", BokehValue::field(&y_col))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("#4C72B0".into())))
        .attr("line_width", BokehValue::value_of(BokehValue::Float(1.5)));

    let line_nonsel_id = id_gen.next();
    let line_nonsel = BokehObject::new("Line", line_nonsel_id)
        .attr("x", BokehValue::field(&filter.column))
        .attr("y", BokehValue::field(&y_col))
        .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

    let renderer = build_glyph_renderer(id_gen, cds.into_value(), line_glyph, Some(line_nonsel), None);

    let range_tool_id = id_gen.next();
    let range_tool = BokehObject::new("RangeTool", range_tool_id)
        .attr("x_range", range_widget.to_ref())
        .attr("overlay", build_range_tool_overlay(id_gen).into_value());

    add_renderers(&mut figure, vec![renderer]);

    // Append RangeTool to toolbar.tools
    let mut range_tool_val = Some(range_tool.into_value());
    for (key, val) in &mut figure.attributes {
        if key == "toolbar" {
            if let BokehValue::Object(tb) = val {
                for (k, v) in &mut tb.attributes {
                    if k == "tools" {
                        if let BokehValue::Array(tools) = v {
                            if let Some(rt) = range_tool_val.take() {
                                tools.push(rt);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(FilterOutput {
        widget: range_widget,
        filter_id: bf_id,
        filter_obj: bf,
        source_key: filter.source_key.clone(),
        switch_label: None,
        is_range_tool: true,
        range_tool_range_id: Some(range_id),
        range_tool_overview: Some(figure),
    })
}

/// Min/max of x-column with 5% buffer on each side. Buffer keeps the draggable
/// window from snapping flush against data edges. Falls back to [start-1, end+1]
/// equivalents if the column has no finite values.
fn compute_bounds(df: &DataFrame, col: &str) -> (f64, f64) {
    let Ok(vals) = get_f64_column(df, col) else {
        return (f64::NEG_INFINITY, f64::INFINITY);
    };
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for v in vals {
        if v.is_finite() {
            if v < lo { lo = v; }
            if v > hi { hi = v; }
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return (f64::NEG_INFINITY, f64::INFINITY);
    }
    let span = hi - lo;
    let buffer = if span > 0.0 { span * 0.05 } else { 1.0 };
    (lo - buffer, hi + buffer)
}

fn build_range_tool_overlay(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("BoxAnnotation", id_gen.next())
        .attr("fill_color", BokehValue::Str("navy".into()))
        .attr("fill_alpha", BokehValue::Float(0.2))
        .attr("line_color", BokehValue::Null)
        .attr("level", BokehValue::Str("underlay".into()))
}
