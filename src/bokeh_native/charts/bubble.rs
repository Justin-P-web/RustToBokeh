//! Bubble chart builder (scatter with data-driven marker size).

use std::collections::HashMap;

use polars::prelude::DataFrame;

use crate::charts::charts::bubble::BubbleConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{
    build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind,
};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::{build_cds_from_entries, get_f64_column, get_str_column};
use super::{add_legend, add_renderers, make_hover_tool, set_axis_labels};

const SIZE_COL: &str = "_bubble_size";
const COLOR_COL: &str = "_bubble_color";

/// Build a bubble-plot `Figure` from a [`BubbleConfig`] and the source frame.
///
/// Marker sizes are computed from `cfg.size_col` and clamped to the
/// configured `size_min`/`size_max` pixel range. `filter_ref` is `Some` when
/// the chart participates in a page-level filter group.
pub fn build_bubble(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &BubbleConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
) -> Result<BokehObject, ChartError> {
    let x_vals = get_f64_column(df, &cfg.x_col).map_err(ChartError::NativeRender)?;
    let y_vals = get_f64_column(df, &cfg.y_col).map_err(ChartError::NativeRender)?;
    let size_raw = get_f64_column(df, &cfg.size_col).map_err(ChartError::NativeRender)?;

    let size_min = cfg.size_min.unwrap_or(8.0);
    let size_max = cfg.size_max.unwrap_or(40.0);
    let sizes_px = map_sizes(&size_raw, size_min, size_max);

    let alpha = cfg.alpha.unwrap_or(0.6);
    let marker = cfg.marker.as_ref().map(|m| m.as_str()).unwrap_or("circle");
    let default_color = cfg.color.as_deref().unwrap_or("#4C72B0");

    let (fill_colors, legend_groups) = match &cfg.color_col {
        Some(col) => {
            let groups_raw = get_str_column(df, col).map_err(ChartError::NativeRender)?;
            let groups = unique_preserve_order(&groups_raw);
            let colors = resolve_palette(cfg.palette.as_ref(), groups.len());
            let map: HashMap<&str, &str> = groups
                .iter()
                .enumerate()
                .map(|(i, g)| (g.as_str(), colors[i].as_str()))
                .collect();
            let per_row: Vec<BokehValue> = groups_raw
                .iter()
                .map(|g| {
                    BokehValue::Str(
                        map.get(g.as_str()).copied().unwrap_or(default_color).to_string(),
                    )
                })
                .collect();
            let legend = groups
                .into_iter()
                .zip(colors)
                .collect::<Vec<_>>();
            (per_row, Some(legend))
        }
        None => {
            let per_row: Vec<BokehValue> = (0..x_vals.len())
                .map(|_| BokehValue::Str(default_color.to_string()))
                .collect();
            (per_row, None)
        }
    };

    let mut default_cols: Vec<&str> = vec![
        cfg.x_col.as_str(),
        cfg.y_col.as_str(),
        cfg.size_col.as_str(),
    ];
    if let Some(c) = &cfg.color_col {
        default_cols.push(c.as_str());
    }
    let ht = make_hover_tool(id_gen, cfg.tooltips.as_ref(), &default_cols);

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::DataRange,
        YRangeKind::DataRange,
        AxisBuilder::x(AxisType::Linear).config(cfg.x_axis.as_ref()),
        AxisBuilder::y(AxisType::Linear).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    // Build CDS from df + derived size/color columns. Preserve all df columns
    // so tooltips using arbitrary columns still work.
    let mut entries: Vec<(String, BokehValue)> = Vec::new();
    for col in df.columns() {
        entries.push((col.name().to_string(), column_to_bokeh_array(col)));
    }
    entries.push((
        SIZE_COL.to_string(),
        BokehValue::Array(sizes_px.iter().map(|&v| BokehValue::Float(v)).collect()),
    ));
    entries.push((COLOR_COL.to_string(), BokehValue::Array(fill_colors)));
    let _ = y_vals; // used via df column serialization
    let _ = x_vals;

    let cds = build_cds_from_entries(id_gen, entries);
    let cds_ref = cds.into_value();

    let glyph = BokehObject::new("Scatter", id_gen.next())
        .attr("x", BokehValue::field(&cfg.x_col))
        .attr("y", BokehValue::field(&cfg.y_col))
        .attr("size", BokehValue::field(SIZE_COL))
        .attr("fill_color", BokehValue::field(COLOR_COL))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str(marker.to_string())));

    let nonsel = BokehObject::new("Scatter", id_gen.next())
        .attr("x", BokehValue::field(&cfg.x_col))
        .attr("y", BokehValue::field(&cfg.y_col))
        .attr("size", BokehValue::field(SIZE_COL))
        .attr("fill_color", BokehValue::field(COLOR_COL))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str(marker.to_string())));

    let renderer = build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), filter_ref);
    add_renderers(&mut figure, vec![renderer]);

    if let Some(groups) = legend_groups {
        if cfg.show_legend.unwrap_or(true) && !groups.is_empty() {
            add_bubble_legend(id_gen, &mut figure, &groups, marker);
        }
    }

    set_axis_labels(&mut figure, &cfg.x_label, &cfg.y_label);
    Ok(figure)
}

/// Sqrt-scale raw values into pixel radii in `[size_min, size_max]`.
///
/// NaN / non-positive values clamp to `size_min`. When all values are equal,
/// every bubble uses the midpoint.
fn map_sizes(raw: &[f64], size_min: f64, size_max: f64) -> Vec<f64> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut pos_count = 0usize;
    let mut valid_count = 0usize;
    for &v in raw {
        if v.is_finite() {
            valid_count += 1;
            if v > 0.0 {
                pos_count += 1;
                if v < lo { lo = v; }
                if v > hi { hi = v; }
            }
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return raw.iter().map(|_| size_min).collect();
    }
    let all_positive_equal = pos_count == valid_count && hi == lo;
    let lo_s = lo.sqrt();
    let hi_s = hi.sqrt();
    let span = hi_s - lo_s;
    raw.iter()
        .map(|&v| {
            if !v.is_finite() || v <= 0.0 {
                size_min
            } else if span <= 0.0 {
                if all_positive_equal {
                    0.5 * (size_min + size_max)
                } else {
                    size_max
                }
            } else {
                let t = (v.sqrt() - lo_s) / span;
                size_min + t * (size_max - size_min)
            }
        })
        .collect()
}

fn unique_preserve_order(vals: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for v in vals {
        if seen.insert(v.clone()) {
            out.push(v.clone());
        }
    }
    out
}

fn add_bubble_legend(
    id_gen: &mut IdGen,
    figure: &mut BokehObject,
    groups: &[(String, String)],
    marker: &str,
) {
    let items: Vec<BokehValue> = groups
        .iter()
        .map(|(label, color)| {
            let swatch = BokehObject::new("Scatter", id_gen.next())
                .attr("x", BokehValue::value_of(BokehValue::Float(0.0)))
                .attr("y", BokehValue::value_of(BokehValue::Float(0.0)))
                .attr("size", BokehValue::value_of(BokehValue::Float(10.0)))
                .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.clone())))
                .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
                .attr(
                    "marker",
                    BokehValue::value_of(BokehValue::Str(marker.to_string())),
                );
            let _ = swatch; // Legend renders from the renderer glyph; keep simple label-only item
            BokehObject::new("LegendItem", id_gen.next())
                .attr("label", BokehValue::value_of(BokehValue::Str(label.clone())))
                .into_value()
        })
        .collect();

    let legend = BokehObject::new("Legend", id_gen.next())
        .attr("items", BokehValue::Array(items))
        .attr("location", BokehValue::Str("top_right".into()))
        .attr("click_policy", BokehValue::Str("hide".into()));
    add_legend(figure, legend);
}

/// Convert a Polars `Column` to a `BokehValue::Array` matching source.rs rules.
/// Inlined from `source::series_to_bokeh_array` (private) — kept minimal and
/// consistent with the rest of the native renderer.
fn column_to_bokeh_array(series: &polars::prelude::Column) -> BokehValue {
    use polars::prelude::DataType;
    match series.dtype() {
        DataType::Float32 => BokehValue::Array(
            series.f32().unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, |x| BokehValue::Float(x as f64)))
                .collect(),
        ),
        DataType::Float64 => BokehValue::Array(
            series.f64().unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, BokehValue::Float))
                .collect(),
        ),
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64
        | DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            let cast = series.cast(&DataType::Int64).unwrap_or_else(|_| series.clone());
            BokehValue::Array(
                cast.i64().unwrap()
                    .into_iter()
                    .map(|v| v.map_or(BokehValue::Null, BokehValue::Int))
                    .collect(),
            )
        }
        DataType::Boolean => BokehValue::Array(
            series.bool().unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, BokehValue::Bool))
                .collect(),
        ),
        DataType::String => BokehValue::Array(
            series.str().unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, |s| BokehValue::Str(s.to_string())))
                .collect(),
        ),
        DataType::Categorical(_, _) | DataType::Enum(_, _) => {
            let cast = series.cast(&DataType::String).unwrap_or_else(|_| series.clone());
            BokehValue::Array(
                cast.str().unwrap()
                    .into_iter()
                    .map(|v| v.map_or(BokehValue::Null, |s| BokehValue::Str(s.to_string())))
                    .collect(),
            )
        }
        _ => {
            let cast = series.cast(&DataType::Float64).unwrap_or_else(|_| series.clone());
            if let Ok(ca) = cast.f64() {
                BokehValue::Array(
                    ca.into_iter()
                        .map(|v| v.map_or(BokehValue::Null, BokehValue::Float))
                        .collect(),
                )
            } else {
                BokehValue::Array(vec![])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};
    use crate::charts::charts::bubble::BubbleConfig;

    fn find_attr<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
        obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn test_df() -> DataFrame {
        df![
            "x" => [1.0, 2.0, 3.0, 4.0],
            "y" => [10.0, 20.0, 15.0, 25.0],
            "pop" => [100.0, 400.0, 900.0, 1600.0],
            "region" => ["N", "S", "N", "E"],
        ].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "t".into(),
            config: ChartConfig::Bubble(
                BubbleConfig::builder()
                    .x("x").y("y").size("pop").x_label("X").y_label("Y")
                    .build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn map_sizes_sqrt_scaled() {
        // raw 100..1600 → sqrt 10..40 → linearly mapped to [8, 40]
        let out = map_sizes(&[100.0, 400.0, 900.0, 1600.0], 8.0, 40.0);
        assert!((out[0] - 8.0).abs() < 1e-9);
        assert!((out[3] - 40.0).abs() < 1e-9);
        assert!(out[1] < out[2] && out[1] > out[0]);
    }

    #[test]
    fn map_sizes_constant_returns_midpoint() {
        let out = map_sizes(&[5.0, 5.0, 5.0], 8.0, 40.0);
        for v in out {
            assert!((v - 24.0).abs() < 1e-9);
        }
    }

    #[test]
    fn map_sizes_nonpositive_clamps_to_min() {
        let out = map_sizes(&[0.0, -1.0, 100.0], 8.0, 40.0);
        assert_eq!(out[0], 8.0);
        assert_eq!(out[1], 8.0);
        assert!((out[2] - 40.0).abs() < 1e-9);
    }

    #[test]
    fn bubble_produces_figure_with_scatter_glyph() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = BubbleConfig::builder()
            .x("x").y("y").size("pop").x_label("X").y_label("Y")
            .build().unwrap();
        let spec = test_spec("Bubble");
        let fig = build_bubble(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        assert_eq!(fig.name, "Figure");
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Scatter"));
        assert!(json.contains(SIZE_COL));
        assert!(json.contains(COLOR_COL));
    }

    #[test]
    fn bubble_color_by_group_adds_legend() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = BubbleConfig::builder()
            .x("x").y("y").size("pop").x_label("X").y_label("Y")
            .color_by("region")
            .build().unwrap();
        let spec = test_spec("Grouped");
        let fig = build_bubble(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Legend"));
        assert!(json.contains("LegendItem"));
    }

    #[test]
    fn bubble_default_color_applied_without_color_col() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = BubbleConfig::builder()
            .x("x").y("y").size("pop").x_label("X").y_label("Y")
            .color("#ff0000")
            .build().unwrap();
        let spec = test_spec("Solid");
        let fig = build_bubble(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("#ff0000"));
    }

    #[test]
    fn bubble_has_hover_tool_and_axis_labels() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = BubbleConfig::builder()
            .x("x").y("y").size("pop").x_label("Revenue").y_label("Profit")
            .build().unwrap();
        let spec = test_spec("Hover");
        let fig = build_bubble(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("HoverTool"));
        assert!(json.contains("Revenue"));
        assert!(json.contains("Profit"));
        // sanity: figure has renderers
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            assert_eq!(arr.len(), 1);
        } else {
            panic!("expected renderers");
        }
    }

    #[test]
    fn bubble_respects_custom_size_range() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = BubbleConfig::builder()
            .x("x").y("y").size("pop").x_label("X").y_label("Y")
            .size_range(4.0, 12.0)
            .build().unwrap();
        let spec = test_spec("SizeRange");
        let fig = build_bubble(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("\"12.0\"") || json.contains("12.0"));
    }
}
