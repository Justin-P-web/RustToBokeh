# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]  — Arrow IPC bytes keyed by source_key (all pages)
#   pages: list[dict]         — each dict has keys:
#       title (str), nav_label (str), slug (str),
#       specs: list[dict] where each spec has:
#           chart_type (str), title (str), source_key (str),
#           x_col (str), value_cols (list[str]), y_label (str),
#           width (int), height (int), indices (list[int] | None)
#   html_template: str        — Jinja2 HTML template source
#   output_dir: str           — directory to write <slug>.html files into

import io
import os

import polars as pl
from bokeh.embed import components
from bokeh.models import CDSView, ColumnDataSource, HoverTool, IndexFilter, Legend, LegendItem
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import dodge, factor_cmap
from jinja2 import Template

_DEFAULT_PALETTE = [
    "#4C72B0", "#DD8452", "#2ca02c",
    "#9467bd", "#e377c2", "#8c564b",
    "#17becf", "#bcbd22",
]

# ── Pre-parse all DataFrames once ────────────────────────────────────────────
# CDS objects are built fresh per page so each page's Bokeh script only
# embeds the data that page actually uses.

_all_dfs = {}
for _key, _raw in frames.items():
    _all_dfs[_key] = pl.read_ipc(io.BytesIO(_raw))


def _build_sources(page_specs):
    """Build fresh ColumnDataSource objects for the source_keys this page uses."""
    sources = {}
    for spec in page_specs:
        key = spec["source_key"]
        if key not in sources:
            df = _all_dfs[key]
            sources[key] = ColumnDataSource({col: df[col].to_list() for col in df.columns})
    return sources


def _make_view(indices):
    """Return a CDSView with an IndexFilter when indices are specified, else None."""
    if indices is None:
        return None
    return CDSView(filter=IndexFilter(indices=list(indices)))


# ── Chart builders ───────────────────────────────────────────────────────────

def build_grouped_bar(spec, source, df):
    """Dodge-based grouped bar chart from a wide-format DataFrame."""
    x_col = spec["x_col"]
    value_cols = spec["value_cols"]
    x_vals = df[x_col].to_list()
    n = len(value_cols)
    bar_width = 0.8 / n
    offsets = [(i - (n - 1) / 2) * bar_width for i in range(n)]
    palette = _DEFAULT_PALETTE[:n]
    view = _make_view(spec["indices"])

    fig = figure(
        x_range=x_vals,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )

    legend_items = []
    for col, offset, color in zip(value_cols, offsets, palette):
        kw = dict(
            x=dodge(x_col, offset, range=fig.x_range),
            top=col,
            width=bar_width * 0.9,
            source=source,
            fill_color=color,
            line_color="white",
        )
        if view is not None:
            kw["view"] = view
        r = fig.vbar(**kw)
        legend_items.append(LegendItem(label=col, renderers=[r]))

    fig.add_layout(Legend(items=legend_items), "right")
    fig.xaxis.major_label_orientation = 1.0
    fig.yaxis.axis_label = spec["y_label"]
    fig.xgrid.grid_line_color = None
    return fig


def build_line_multi(spec, source, df):
    """One line per value column, sharing the same ColumnDataSource.

    CDSView/IndexFilter is incompatible with connected glyphs (E-1024), so
    index filtering is handled differently per glyph type:
      - Line:    restrict figure x_range to the filtered x values; Bokeh only
                 renders segments whose endpoints are within the categorical range.
      - Scatter: apply CDSView+IndexFilter normally (discrete glyph, no issue).
    """
    x_col = spec["x_col"]
    value_cols = spec["value_cols"]
    x_vals = df[x_col].to_list()
    palette = _DEFAULT_PALETTE[:len(value_cols)]
    indices = spec["indices"]

    if indices is not None:
        display_x = [x_vals[i] for i in indices]
        scatter_view = CDSView(filter=IndexFilter(indices=list(indices)))
    else:
        display_x = x_vals
        scatter_view = None

    fig = figure(
        x_range=display_x,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )

    legend_items = []
    for col, color in zip(value_cols, palette):
        r = fig.line(x=x_col, y=col, source=source, line_color=color, line_width=2)
        scatter_kw = dict(x=x_col, y=col, source=source, fill_color=color, size=6, line_color="white")
        if scatter_view is not None:
            scatter_kw["view"] = scatter_view
        fig.scatter(**scatter_kw)
        legend_items.append(LegendItem(label=col, renderers=[r]))

    fig.add_layout(Legend(items=legend_items), "right")
    fig.xaxis.major_label_orientation = 0.8
    fig.yaxis.axis_label = spec["y_label"]
    return fig


def build_hbar(spec, source, df):
    """Horizontal bar chart; x_col is the category column (rendered on y-axis)."""
    x_col = spec["x_col"]
    value_col = spec["value_cols"][0]
    categories = df[x_col].to_list()
    palette = _DEFAULT_PALETTE[:len(categories)]
    view = _make_view(spec["indices"])

    fig = figure(
        y_range=categories,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )

    kw = dict(
        y=x_col,
        right=value_col,
        height=0.6,
        source=source,
        fill_color=factor_cmap(x_col, palette=palette, factors=categories),
        line_color="white",
    )
    if view is not None:
        kw["view"] = view
    fig.hbar(**kw)

    fig.xaxis.axis_label = spec["y_label"]
    fig.ygrid.grid_line_color = None
    return fig


def build_scatter_plot(spec, source, df):
    """Numeric x/y scatter; x_col is the x-axis column, value_cols[0] is y."""
    x_col = spec["x_col"]
    y_col = spec["value_cols"][0]
    view = _make_view(spec["indices"])

    # Build hover showing all CDS columns; brace-quote names for safety.
    hover = HoverTool(tooltips=[(col, f"@{{{col}}}") for col in df.columns])

    fig = figure(
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools=[hover, "pan", "wheel_zoom", "box_zoom", "reset", "save"],
    )

    kw = dict(
        x=x_col,
        y=y_col,
        source=source,
        size=10,
        fill_color=_DEFAULT_PALETTE[0],
        fill_alpha=0.75,
        line_color="white",
    )
    if view is not None:
        kw["view"] = view
    fig.scatter(**kw)

    fig.xaxis.axis_label = x_col
    fig.yaxis.axis_label = spec["y_label"]
    return fig


# ── Dispatch table ───────────────────────────────────────────────────────────

_BUILDERS = {
    "grouped_bar": build_grouped_bar,
    "line_multi":  build_line_multi,
    "hbar":        build_hbar,
    "scatter_plot": build_scatter_plot,
}

# ── Render one HTML file per page ────────────────────────────────────────────

bokeh_js_url = CDN.js_files[0]
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""
template = Template(html_template)

# Navigation entries shared across all pages.
nav_pages = [{"label": p["nav_label"], "href": p["slug"] + ".html"} for p in pages]

os.makedirs(output_dir, exist_ok=True)

for page in pages:
    # Build fresh CDS objects scoped to this page's source_keys only.
    # components() will then serialize only the data those figures reference.
    sources = _build_sources(page["specs"])

    figures_list = []
    for spec in page["specs"]:
        builder = _BUILDERS.get(spec["chart_type"])
        if builder is None:
            raise ValueError(f"Unknown chart_type: {spec['chart_type']!r}")
        key = spec["source_key"]
        figures_list.append(builder(spec, sources[key], _all_dfs[key]))

    script, divs = components(figures_list)

    plots = [
        {"title": spec["title"], "div": div, "width": spec["width"]}
        for spec, div in zip(page["specs"], divs)
    ]

    # Mark the current page as active in the nav bar.
    this_nav = [
        {**entry, "active": entry["href"] == page["slug"] + ".html"}
        for entry in nav_pages
    ]

    html = template.render(
        title=page["title"],
        nav_pages=this_nav,
        bokeh_js_url=bokeh_js_url,
        bokeh_css_url=bokeh_css_url,
        plot_script=script,
        plots=plots,
    )

    out_path = os.path.join(output_dir, page["slug"] + ".html")
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"  wrote {out_path}")
