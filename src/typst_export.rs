//! Typst export — pure-Rust lowering of a Dashboard to a Typst source
//! document plus a `resources/` directory of SVG chart images.
//!
//! The HTML render path produces an interactive dashboard; this path produces
//! a static, printable companion. Run `typst compile report.typ` afterward to
//! produce a PDF.
//!
//! # Output layout
//!
//! ```text
//! <output_dir>/
//!     report.typ                  // text source; references images from resources/
//!     resources/
//!         chart_001_<slug>.svg    // one SVG per supported chart
//!         chart_002_<slug>.svg
//!         ...
//! ```
//!
//! [`TypstReport::write_to`] writes both the `.typ` file and the `resources/`
//! directory in one call. [`TypstReport::source_inlined`] returns an alternative
//! standalone variant of the source with each chart image embedded inline via
//! `image.decode`, suitable for client-side download where shipping a folder
//! is awkward.
//!
//! # Coverage (v3)
//!
//! - **Paragraph, Table, StatGrid** — fully lowered to native Typst markup.
//! - **Line, Scatter, Bubble** — rendered as SVG plots written to
//!   `resources/`. Each chart gets a deterministic numbered filename.
//! - **HBar, GroupedBar, Histogram, Pie, BoxPlot, Density** — labeled
//!   placeholder block plus a numeric min/mean/max summary table where the
//!   value columns are numeric. Real plot rendering is planned.
//! - **Filter widgets** — omitted (interactive only).

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

use polars::prelude::*;

use crate::charts::customization::palette::PaletteSpec;
use crate::charts::{
    BoxPlotConfig, BubbleConfig, ChartConfig, ChartSpec, DensityConfig, GroupedBarConfig,
    HBarConfig, HistogramConfig, HistogramDisplay, LineConfig, PieConfig, ScatterConfig,
};
use crate::modules::{ColumnFormat, PageModule, ParagraphSpec, StatGridSpec, TableSpec};
use crate::pages::Page;

/// Color palette used for plot series. Mirrors the seaborn-style defaults
/// elsewhere in the crate so static and interactive views look consistent.
const PLOT_PALETTE: [&str; 10] = [
    "#4C72B0", "#DD8452", "#55A467", "#C44E52", "#8172B2", "#937860", "#DA8BC3", "#8C8C8C",
    "#CCB974", "#64B5CD",
];

/// Final rendered width of a chart image inside the Typst document.
const CHART_DISPLAY_WIDTH: &str = "14cm";

/// A single binary resource that lives next to the `report.typ` file.
///
/// `relative_path` is forward-slash separated and rooted at the same directory
/// as `report.typ` (e.g. `"resources/chart_001.svg"`).
#[derive(Clone, Debug)]
pub struct TypstResource {
    /// Path relative to the `report.typ` file, using forward slashes.
    pub relative_path: String,
    /// Raw bytes to write at that path.
    pub bytes: Vec<u8>,
}

/// Bundled output of [`build_typst_report`]: the Typst source plus every
/// resource file it references.
#[derive(Clone, Debug)]
pub struct TypstReport {
    /// Typst source. Image references use relative paths into
    /// [`resources`](Self::resources) (e.g. `#image("resources/chart_001.svg")`).
    pub source: String,
    /// Image and other binary files referenced by [`source`](Self::source).
    pub resources: Vec<TypstResource>,
}

impl TypstReport {
    /// Write `report.typ` and every resource file under `dir`. Creates
    /// intermediate directories (including `resources/`) as needed.
    ///
    /// # Errors
    ///
    /// Returns the first I/O error encountered while creating directories or
    /// writing files.
    pub fn write_to(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        std::fs::write(dir.join("report.typ"), &self.source)?;
        for r in &self.resources {
            let rel: &Path = Path::new(&r.relative_path);
            let full = dir.join(rel);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(full, &r.bytes)?;
        }
        Ok(())
    }

    /// Return a standalone variant of the source where every
    /// `#image("resources/<file>", ...)` reference is replaced with an inline
    /// `#image(bytes("<svg…>"), format: "svg", ...)` call carrying the file's
    /// contents.
    ///
    /// Useful when the source must be transmitted as a single text payload
    /// (e.g. the in-page download button).
    #[must_use]
    pub fn source_inlined(&self) -> String {
        let mut out = self.source.clone();
        for r in &self.resources {
            let needle = format!("#image(\"{}\"", r.relative_path);
            let svg = std::str::from_utf8(&r.bytes).unwrap_or("");
            let escaped = esc_str(svg);
            let replacement = format!("#image(bytes(\"{}\"), format: \"svg\"", escaped);
            out = out.replace(&needle, &replacement);
        }
        out
    }
}

/// Build the Typst report.
///
/// `frames` carries the same `DataFrame`s used by the HTML render path, keyed
/// by `source_key`. Pages render in insertion order, each as its own Typst
/// section separated by `#pagebreak()`. Charts that this exporter knows how
/// to plot are rendered to SVG and returned in
/// [`TypstReport::resources`](TypstReport::resources); other charts emit a
/// placeholder block plus a min/mean/max summary table.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use rust_to_bokeh::typst_export::build_typst_report;
///
/// let report = build_typst_report(&dash.pages, &frames, "Quarterly Report");
/// report.write_to(Path::new("./out"))?;
/// // typst compile out/report.typ
/// ```
#[must_use]
pub fn build_typst_report(
    pages: &[Page],
    frames: &HashMap<String, DataFrame>,
    title: &str,
) -> TypstReport {
    let mut ctx = LowerCtx {
        out: String::new(),
        resources: Vec::new(),
        chart_idx: 0,
        frames,
    };

    write_preamble(&mut ctx.out, title);

    if !title.is_empty() {
        write_title_page(&mut ctx.out, title);
    }

    write_contents_page(&mut ctx.out);

    for (i, page) in pages.iter().enumerate() {
        if i > 0 {
            ctx.out.push_str("#pagebreak()\n\n");
        }

        let template = match page.category.as_deref() {
            Some("Summary") => "summary-page",
            Some("Tests") => "test-page",
            _ => "",
        };

        if !template.is_empty() {
            writeln!(
                ctx.out,
                "#{template}(\"{}\")[",
                esc_str(&page.title),
            )
            .unwrap();
        } else if let Some(cat) = &page.category {
            writeln!(
                ctx.out,
                "#category-page(\"{}\", \"{}\")[",
                esc_str(cat),
                esc_str(&page.title),
            )
            .unwrap();
        } else {
            writeln!(
                ctx.out,
                "#plain-page(\"{}\")[",
                esc_str(&page.title),
            )
            .unwrap();
        }

        let mut order: Vec<usize> = (0..page.modules.len()).collect();
        order.sort_by_key(|&k| {
            let g = match &page.modules[k] {
                PageModule::Chart(s) => &s.grid,
                PageModule::Paragraph(s) => &s.grid,
                PageModule::Table(s) => &s.grid,
                PageModule::StatGrid(s) => &s.grid,
            };
            (g.row, g.col)
        });

        for &k in &order {
            lower_module(&mut ctx, &page.modules[k]);
            ctx.out.push('\n');
        }
        // Close the page-template content block and attach a slug label so
        // table cells (and any future cross-references) can `#link` to it.
        ctx.out.push_str("]\n");
        if !page.slug.is_empty() {
            writeln!(
                ctx.out,
                "<page-{}>",
                sanitize_label(&page.slug),
            )
            .unwrap();
        }
        ctx.out.push('\n');
    }

    TypstReport {
        source: ctx.out,
        resources: ctx.resources,
    }
}

// ── Document scaffolding ─────────────────────────────────────────────────────

/// Emit the document preamble: design tokens (color, spacing), page setup with
/// running header + footer, typography rules, table styling, and reusable
/// content helpers (`eyebrow`, `accent-rule`, `muted-note`).
fn write_preamble(out: &mut String, title: &str) {
    out.push_str("// RustToBokeh — generated Typst report.\n");
    out.push_str("// Compile with: typst compile report.typ\n\n");

    // Design tokens.
    out.push_str("#let accent = oklch(45%, 0.13, 250deg)\n");
    out.push_str("#let accent-dim = oklch(72%, 0.05, 250deg)\n");
    out.push_str("#let ink = luma(25)\n");
    out.push_str("#let mute = luma(110)\n");
    out.push_str("#let whisper = luma(170)\n");
    out.push_str("#let rule-color = luma(220)\n");
    out.push_str("#let space = (xs: 0.4em, sm: 0.7em, md: 1.1em, lg: 1.8em, xl: 2.6em)\n\n");

    // Reusable inline helpers.
    out.push_str("#let eyebrow(body) = text(size: 7pt, fill: accent-dim, tracking: 0.14em, weight: \"semibold\")[#upper[#body]]\n");
    out.push_str("#let muted-note(body) = text(size: 8.5pt, fill: mute, style: \"italic\", body)\n");
    out.push_str("#let accent-rule = line(length: 3em, stroke: 2pt + accent)\n\n");

    let header_title = esc_str(title);
    writeln!(out, "#let report-title = \"{}\"", header_title).unwrap();
    out.push('\n');

    // Page setup with running header on inner pages and a tiny footer.
    out.push_str(
        "#set page(\n  \
         paper: \"us-letter\",\n  \
         margin: (x: 0.95in, top: 1.05in, bottom: 0.95in),\n  \
         header: context {\n    \
         if counter(page).get().first() <= 1 { return }\n    \
         set text(size: 8pt, fill: mute)\n    \
         grid(\n      \
         columns: (1fr, auto),\n      \
         align: (left + bottom, right + bottom),\n      \
         report-title,\n      \
         [Page #counter(page).display(\"1\")],\n    \
         )\n    \
         v(-0.3em)\n    \
         line(length: 100%, stroke: 0.4pt + rule-color)\n  \
         },\n  \
         footer: none,\n\
         )\n\n",
    );

    // Typography.
    out.push_str("#set text(size: 10pt, fill: ink)\n");
    out.push_str("#set par(leading: 0.65em, justify: true, first-line-indent: 0pt)\n");
    out.push_str("#set heading(numbering: none)\n");
    out.push_str(
        "#show heading.where(level: 1): it => {\n  \
         set block(above: 0pt, below: space.md)\n  \
         text(size: 26pt, weight: \"bold\", fill: ink, it.body)\n  \
         v(space.xs)\n  \
         accent-rule\n\
         }\n",
    );
    out.push_str(
        "#show heading.where(level: 2): it => block(above: space.lg, below: space.sm)[\n  \
         #text(size: 12pt, weight: \"semibold\", fill: ink, it.body)\n\
         ]\n\n",
    );

    // Table styling: subtle zebra stripes, accent rule under header row only.
    out.push_str(
        "#set table(\n  \
         stroke: (x, y) => if y == 0 { (bottom: 0.8pt + accent) } else { none },\n  \
         inset: (x: 8pt, y: 6pt),\n  \
         fill: (x, y) => if y == 0 { luma(245) } else if calc.even(y) { luma(252) } else { none },\n  \
         align: left,\n\
         )\n",
    );
    out.push_str(
        "#show table.cell.where(y: 0): set text(weight: \"semibold\", fill: ink)\n\n",
    );

    // ── Page templates ──────────────────────────────────────────────────────
    //
    // Two distinct layouts so cross-test summary pages and per-test detail
    // pages read as members of the same family but visually differentiated.
    out.push_str(
        "#let summary-page(title, body) = {\n  \
         v(space.md)\n  \
         eyebrow[Sensor Summary]\n  \
         v(space.xs)\n  \
         heading(level: 1, outlined: true)[#title]\n  \
         body\n\
         }\n",
    );
    out.push_str(
        "#let test-page(title, body) = {\n  \
         v(space.md)\n  \
         grid(columns: (4pt, 1fr), column-gutter: 0.8em,\n    \
         box(width: 4pt, height: 1.6em, fill: accent),\n    \
         eyebrow[Test Case],\n  \
         )\n  \
         v(space.xs)\n  \
         heading(level: 1, outlined: true)[#title]\n  \
         body\n\
         }\n",
    );
    // Generic page (no eyebrow) — used when the page has no category or its
    // category does not match a recognised template.
    out.push_str(
        "#let plain-page(title, body) = {\n  \
         v(space.md)\n  \
         heading(level: 1, outlined: true)[#title]\n  \
         body\n\
         }\n",
    );
    out.push_str(
        "#let category-page(category, title, body) = {\n  \
         v(space.md)\n  \
         eyebrow[#category]\n  \
         v(space.xs)\n  \
         heading(level: 1, outlined: true)[#title]\n  \
         body\n\
         }\n\n",
    );
}

/// Emit the title page: asymmetric, left-aligned, vertically centered, with
/// eyebrow / title / accent bar / colophon stack.
fn write_title_page(out: &mut String, title: &str) {
    out.push_str("#page(header: none, footer: none)[\n");
    out.push_str("  #set align(left + horizon)\n");
    out.push_str("  #set par(justify: false)\n");
    out.push_str("  #pad(right: 1.6in)[\n");
    out.push_str("    #eyebrow[Report]\n");
    out.push_str("    #v(space.sm)\n");
    writeln!(
        out,
        "    #text(size: 38pt, weight: \"bold\", fill: ink, \"{}\")",
        esc_str(title),
    )
    .unwrap();
    out.push_str("    #v(space.md)\n");
    out.push_str("    #block(width: 5em, height: 3pt, fill: accent)\n");
    out.push_str("    #v(space.xl)\n");
    out.push_str("    #muted-note[Generated by RustToBokeh]\n");
    out.push_str("  ]\n");
    out.push_str("]\n\n");
}

/// Emit a dedicated contents page mirroring the section-heading typography
/// (eyebrow / title / accent rule) so it visually belongs to the same family.
fn write_contents_page(out: &mut String) {
    out.push_str("#eyebrow[Contents]\n");
    out.push_str("#v(space.xs)\n");
    out.push_str("#text(size: 26pt, weight: \"bold\", fill: ink, \"Sections\")\n");
    out.push_str("#v(space.xs)\n");
    out.push_str("#accent-rule\n");
    out.push_str("#v(space.lg)\n");
    out.push_str("#outline(title: none, depth: 2, indent: 1em)\n\n");
    out.push_str("#pagebreak()\n\n");
}

// ── Lowering ─────────────────────────────────────────────────────────────────

struct LowerCtx<'a> {
    out: String,
    resources: Vec<TypstResource>,
    chart_idx: usize,
    frames: &'a HashMap<String, DataFrame>,
}

fn lower_module(ctx: &mut LowerCtx, module: &PageModule) {
    match module {
        PageModule::Paragraph(p) => lower_paragraph(&mut ctx.out, p),
        PageModule::Table(t) => match ctx.frames.get(&t.source_key) {
            Some(df) => lower_table(&mut ctx.out, t, df),
            None => {
                writeln!(
                    ctx.out,
                    "#block(stroke: 0.5pt, inset: 8pt)[_Missing data for table source `{}`_]",
                    esc_markup(&t.source_key),
                )
                .unwrap();
            }
        },
        PageModule::StatGrid(g) => lower_stat_grid(&mut ctx.out, g),
        PageModule::Chart(c) => lower_chart(ctx, c),
    }
}

fn lower_paragraph(out: &mut String, p: &ParagraphSpec) {
    if let Some(title) = &p.title {
        writeln!(out, "== {}", esc_markup(title)).unwrap();
        out.push('\n');
    }
    for para in p.text.split("\n\n") {
        let trimmed = para.trim();
        if !trimmed.is_empty() {
            out.push_str(&esc_markup(trimmed));
            out.push_str("\n\n");
        }
    }
}

fn lower_stat_grid(out: &mut String, g: &StatGridSpec) {
    if g.items.is_empty() {
        return;
    }
    let n = g.items.len();
    // Cap at 4 columns so 6+ items wrap to a second row instead of forcing
    // each column to shrink and re-introduce mid-word breaks.
    let cols = n.min(4);
    out.push_str("#block(above: space.md, below: space.md)[\n");
    out.push_str("  #set par(justify: false)\n");
    out.push_str("  #set text(hyphenate: false)\n");
    writeln!(
        out,
        "  #grid(\n    \
         columns: {} * (auto,),\n    \
         row-gutter: space.md,\n    \
         align: top + left,\n    \
         stroke: (x, y) => if x > 0 {{ (left: 0.4pt + rule-color) }} else {{ none }},\n    \
         inset: (x, y) => if x == 0 {{ (left: 0pt, right: 1.2em, y: 0pt) }} else {{ (x: 1.2em, y: 0pt) }},",
        cols,
    )
    .unwrap();
    for item in &g.items {
        out.push_str("    [\n");
        writeln!(
            out,
            "      #eyebrow[{}]",
            esc_markup(&item.label),
        )
        .unwrap();
        out.push_str("      #v(0.35em)\n");
        // Replace spaces with NBSP so multi-word values ("Flow Rate, Pressure")
        // never break across lines; auto columns grow to fit.
        let value_nb = item.value.replace(' ', "\u{00A0}");
        match &item.suffix {
            Some(suffix) if !suffix.is_empty() => {
                let suffix_nb = suffix.replace(' ', "\u{00A0}");
                writeln!(
                    out,
                    "      #text(size: 18pt, weight: \"semibold\", fill: ink, \"{}\")#text(size: 9pt, fill: mute, \"\u{00A0}{}\")",
                    esc_str(&value_nb),
                    esc_str(&suffix_nb),
                )
                .unwrap()
            }
            _ => writeln!(
                out,
                "      #text(size: 18pt, weight: \"semibold\", fill: ink, \"{}\")",
                esc_str(&value_nb),
            )
            .unwrap(),
        }
        out.push_str("    ],\n");
    }
    out.push_str("  )\n");
    out.push_str("]\n");
}

fn lower_table(out: &mut String, t: &TableSpec, df: &DataFrame) {
    if t.columns.is_empty() {
        return;
    }
    if !t.title.is_empty() {
        writeln!(out, "== {}", esc_markup(&t.title)).unwrap();
        out.push('\n');
    }

    let n_cols = t.columns.len();
    // Per-column sizing & alignment driven by `ColumnFormat`: text columns
    // size to content and stay left-aligned, numeric columns share the
    // remaining horizontal space (so the table fills the page width) and
    // right-align so digits line up.
    let mut col_widths: Vec<&str> = Vec::with_capacity(n_cols);
    let mut col_aligns: Vec<&str> = Vec::with_capacity(n_cols);
    for c in &t.columns {
        match c.format {
            ColumnFormat::Text => {
                col_widths.push("auto");
                col_aligns.push("left");
            }
            _ => {
                col_widths.push("1fr");
                col_aligns.push("right");
            }
        }
    }
    // All-text tables would otherwise hug the left edge — promote everything
    // after the first column to `1fr` so the table still stretches.
    if col_widths.iter().all(|w| *w == "auto") && n_cols > 1 {
        for i in 1..n_cols {
            col_widths[i] = "1fr";
        }
    }
    writeln!(out, "#table(").unwrap();
    writeln!(out, "  columns: ({},),", col_widths.join(", ")).unwrap();
    writeln!(out, "  align: ({},),", col_aligns.join(", ")).unwrap();
    out.push_str("  table.header(");
    for (i, col) in t.columns.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        write!(out, "[*{}*]", esc_markup(&col.label)).unwrap();
    }
    out.push_str("),\n");

    for row in 0..df.height() {
        out.push_str("  ");
        for (i, col_def) in t.columns.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let cell = match df.column(&col_def.key) {
                Ok(series) => format_cell(series, row, &col_def.format),
                Err(_) => String::new(),
            };
            write!(out, "[{}]", render_cell(&cell)).unwrap();
        }
        out.push_str(",\n");
    }
    out.push_str(")\n");
}

/// Lower a cell's raw text into Typst content. Detects `<a href="…">…</a>`
/// anchors (left over from the HTML render path) and converts them to either
/// an internal cross-reference (`#link(<page-…>)[…]`) when the URL is a page
/// slug, or an external hyperlink (`#link("…")[…]`) otherwise. Plain text
/// falls through `esc_markup`.
fn render_cell(cell: &str) -> String {
    if let Some((href, label)) = parse_anchor(cell) {
        let label_md = esc_markup(label);
        if let Some(stem) = href.strip_suffix(".html") {
            return format!(
                "#link(<page-{}>)[{}]",
                sanitize_label(stem),
                label_md,
            );
        }
        return format!("#link(\"{}\")[{}]", esc_str(href), label_md);
    }
    esc_markup(cell)
}

fn lower_chart(ctx: &mut LowerCtx, spec: &ChartSpec) {
    if !spec.title.is_empty() {
        writeln!(ctx.out, "== {}", esc_markup(&spec.title)).unwrap();
        ctx.out.push('\n');
    }
    let kind = spec.config.chart_type_str();
    let df = ctx.frames.get(&spec.source_key);
    let rows = df.map(DataFrame::height).unwrap_or(0);

    let svg = df.and_then(|d| build_chart_svg(spec, ctx.frames, d));

    match svg {
        Some(svg_text) => {
            ctx.chart_idx += 1;
            let slug = sanitize_slug(&spec.title);
            let filename = if slug.is_empty() {
                format!("chart_{:03}.svg", ctx.chart_idx)
            } else {
                format!("chart_{:03}_{}.svg", ctx.chart_idx, slug)
            };
            let rel = format!("resources/{}", filename);

            ctx.resources.push(TypstResource {
                relative_path: rel.clone(),
                bytes: svg_text.into_bytes(),
            });

            ctx.out
                .push_str("#block(width: 100%, breakable: false)[\n");
            writeln!(
                ctx.out,
                "  #eyebrow[Figure {:02} · {}]",
                ctx.chart_idx,
                esc_markup(kind),
            )
            .unwrap();
            ctx.out.push_str("  #v(space.xs)\n");
            writeln!(
                ctx.out,
                "  #align(center)[#image(\"{}\", width: {})]",
                rel, CHART_DISPLAY_WIDTH,
            )
            .unwrap();
            ctx.out.push_str("  #v(space.xs)\n");
            writeln!(
                ctx.out,
                "  #align(center)[#muted-note[Source: {} · {} observations]]",
                esc_markup(&spec.source_key),
                rows,
            )
            .unwrap();
            ctx.out.push_str("]\n");
        }
        None => {
            // No SVG could be rendered — emit a quiet stub block instead of a
            // loud gray placeholder so the layout rhythm is preserved.
            ctx.out
                .push_str("#block(width: 100%, inset: (y: space.sm))[\n");
            writeln!(
                ctx.out,
                "  #eyebrow[Figure {:02} · {}]",
                ctx.chart_idx + 1,
                esc_markup(kind),
            )
            .unwrap();
            ctx.out.push_str("  #v(space.xs)\n");
            writeln!(
                ctx.out,
                "  #muted-note[No data available · source: {}]",
                esc_markup(&spec.source_key),
            )
            .unwrap();
            ctx.out.push_str("]\n");
        }
    }

    if let Some(d) = df {
        let summary = chart_summary(spec, d);
        if !summary.is_empty() {
            ctx.out.push_str("#v(space.sm)\n");
            ctx.out.push_str(&summary);
            ctx.out.push('\n');
        }
    }
}

/// Sanitize a string into a Typst label name (a-z, 0-9, `-`, `_`, `.`, `:`).
/// The `page-` prefix added at call sites guarantees the result starts with
/// a letter even when the slug begins with a digit.
fn sanitize_label(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | ':' => out.push(c),
            _ => out.push('-'),
        }
    }
    out
}

/// If a table cell contains exactly an HTML `<a href="...">label</a>` anchor,
/// return the `(href, label)` pair. URLs ending in `.html` are treated as
/// page slugs and converted to internal Typst cross-references at the call
/// site; everything else is rendered as a plain hyperlink.
fn parse_anchor(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    let after_open = s.strip_prefix("<a href=\"")?;
    let (href, rest) = after_open.split_once("\">")?;
    let label = rest.strip_suffix("</a>")?;
    if href.is_empty() || label.is_empty() {
        return None;
    }
    Some((href, label))
}

/// Produce a filesystem-safe slug from a chart title. Lowercases, replaces
/// non-alphanumeric runs with `_`, and trims to 32 chars.
fn sanitize_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_sep = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.len() > 32 {
        out.truncate(32);
        while out.ends_with('_') {
            out.pop();
        }
    }
    out
}

// ── SVG plot dispatch ────────────────────────────────────────────────────────

fn build_chart_svg(spec: &ChartSpec, frames: &HashMap<String, DataFrame>, df: &DataFrame) -> Option<String> {
    match &spec.config {
        ChartConfig::Line(c) => build_line_svg(c, df),
        ChartConfig::Scatter(c) => build_scatter_svg(c, df),
        ChartConfig::Bubble(c) => build_bubble_svg(c, df),
        ChartConfig::HBar(c) => build_hbar_svg(c, df),
        ChartConfig::GroupedBar(c) => build_grouped_bar_svg(c, df),
        ChartConfig::Histogram(c) => build_histogram_svg(c, df),
        ChartConfig::Pie(c) => build_pie_svg(c, df),
        ChartConfig::BoxPlot(c) => build_box_plot_svg(c, df, frames),
        ChartConfig::Density(c) => build_density_svg(c, df),
    }
}

fn build_line_svg(c: &LineConfig, df: &DataFrame) -> Option<String> {
    let (xs, x_ticks) = numeric_or_categorical_x(df, &c.x_col)?;
    if xs.is_empty() {
        return None;
    }
    let mut series = Vec::new();
    for (i, ycol) in c.y_cols.iter().enumerate() {
        let Some(ys) = extract_f64(df, ycol) else {
            continue;
        };
        let points: Vec<(f64, f64)> = xs.iter().copied().zip(ys.iter().copied()).collect();
        let color = PLOT_PALETTE[i % PLOT_PALETTE.len()].to_string();
        series.push(svg_plot::Series {
            label: Some(ycol.clone()),
            color,
            points,
            kind: svg_plot::SeriesKind::Line {
                width: c.line_width.unwrap_or(2.0),
                marker_radius: c.point_size.unwrap_or(7.0) / 2.0,
            },
        });
    }
    if series.is_empty() {
        return None;
    }
    Some(svg_plot::render(&svg_plot::Plot {
        series,
        x_label: c.x_col.clone(),
        y_label: c.y_label.clone(),
        x_ticks,
    }))
}

/// Try to extract a column as `f64`. If that fails (e.g. the column is a
/// string), fall back to using positional indices `0..n-1` and return the
/// string values as explicit X tick labels for the plot renderer.
fn numeric_or_categorical_x(
    df: &DataFrame,
    col: &str,
) -> Option<(Vec<f64>, Option<Vec<(f64, String)>>)> {
    if let Some(xs) = extract_f64(df, col) {
        return Some((xs, None));
    }
    let labels = extract_str(df, col)?;
    if labels.is_empty() {
        return None;
    }
    let xs: Vec<f64> = (0..labels.len()).map(|i| i as f64).collect();
    // Subsample tick labels when too many (>12) to avoid label overlap.
    let stride = (labels.len() / 12).max(1);
    let ticks: Vec<(f64, String)> = labels
        .iter()
        .enumerate()
        .filter(|(i, _)| i % stride == 0 || *i == labels.len() - 1)
        .map(|(i, l)| (i as f64, l.clone()))
        .collect();
    Some((xs, Some(ticks)))
}

fn build_scatter_svg(c: &ScatterConfig, df: &DataFrame) -> Option<String> {
    let (xs, x_ticks) = numeric_or_categorical_x(df, &c.x_col)?;
    let ys = extract_f64(df, &c.y_col)?;
    let points: Vec<(f64, f64)> = xs.iter().copied().zip(ys.iter().copied()).collect();
    if points.is_empty() {
        return None;
    }
    let color = c.color.clone().unwrap_or_else(|| "#4C72B0".to_string());
    let series = vec![svg_plot::Series {
        label: None,
        color,
        points,
        kind: svg_plot::SeriesKind::Scatter {
            radius: c.marker_size.unwrap_or(10.0) / 2.0,
            alpha: c.alpha.unwrap_or(0.7),
        },
    }];
    Some(svg_plot::render(&svg_plot::Plot {
        series,
        x_label: c.x_label.clone(),
        y_label: c.y_label.clone(),
        x_ticks,
    }))
}

fn build_bubble_svg(c: &BubbleConfig, df: &DataFrame) -> Option<String> {
    let xs = extract_f64(df, &c.x_col)?;
    let ys = extract_f64(df, &c.y_col)?;
    let raw_sizes = extract_f64(df, &c.size_col)?;
    if xs.is_empty() || ys.is_empty() || raw_sizes.is_empty() {
        return None;
    }
    let smin = c.size_min.unwrap_or(8.0);
    let smax = c.size_max.unwrap_or(40.0);
    let max_size = raw_sizes
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .fold(0.0_f64, f64::max)
        .max(1e-9);
    let radii: Vec<f64> = raw_sizes
        .iter()
        .map(|&v| {
            if !v.is_finite() || v <= 0.0 {
                smin / 2.0
            } else {
                let frac = (v / max_size).sqrt();
                (smin + frac * (smax - smin)) / 2.0
            }
        })
        .collect();
    let points: Vec<(f64, f64)> = xs.iter().copied().zip(ys.iter().copied()).collect();

    let color = c.color.clone().unwrap_or_else(|| "#4C72B0".to_string());
    let series = vec![svg_plot::Series {
        label: None,
        color,
        points,
        kind: svg_plot::SeriesKind::Bubble {
            radii,
            alpha: c.alpha.unwrap_or(0.6),
        },
    }];
    Some(svg_plot::render(&svg_plot::Plot {
        series,
        x_label: c.x_label.clone(),
        y_label: c.y_label.clone(),
        x_ticks: None,
    }))
}

fn build_hbar_svg(c: &HBarConfig, df: &DataFrame) -> Option<String> {
    let cats = extract_str(df, &c.category_col)?;
    let vals = extract_f64(df, &c.value_col)?;
    if cats.is_empty() || vals.is_empty() {
        return None;
    }
    let color = c.color.clone().unwrap_or_else(|| "#4C72B0".to_string());
    Some(svg_plot::render_hbar(&cats, &vals, &color, &c.x_label))
}

fn build_grouped_bar_svg(c: &GroupedBarConfig, df: &DataFrame) -> Option<String> {
    let xs = extract_str(df, &c.x_col)?;
    let groups = extract_str(df, &c.group_col)?;
    let vals = extract_f64(df, &c.value_col)?;
    if xs.is_empty() || groups.is_empty() || vals.is_empty() {
        return None;
    }
    let palette = resolve_palette(c.palette.as_ref(), 10);
    Some(svg_plot::render_grouped_bar(
        &xs,
        &groups,
        &vals,
        &palette,
        &c.y_label,
        c.bar_width.unwrap_or(0.9),
    ))
}

fn build_histogram_svg(c: &HistogramConfig, df: &DataFrame) -> Option<String> {
    let lefts = extract_f64(df, "left")?;
    let rights = extract_f64(df, "right")?;
    let display = c.display.clone().unwrap_or(HistogramDisplay::Count);
    let value_col = match display {
        HistogramDisplay::Count => "count",
        HistogramDisplay::Pdf => "pdf",
        HistogramDisplay::Cdf => "cdf",
    };
    let values = extract_f64(df, value_col)?;
    if lefts.is_empty() || values.is_empty() {
        return None;
    }
    let color = c.color.clone().unwrap_or_else(|| "#4C72B0".to_string());
    let alpha = c.alpha.unwrap_or(0.7);
    let y_label = c.y_label.clone().unwrap_or_else(|| match display {
        HistogramDisplay::Count => "Count".into(),
        HistogramDisplay::Pdf => "Density".into(),
        HistogramDisplay::Cdf => "Cumulative Fraction".into(),
    });
    let is_cdf = matches!(display, HistogramDisplay::Cdf);
    Some(svg_plot::render_histogram(
        &lefts, &rights, &values, is_cdf, &color, alpha, &c.x_label, &y_label,
    ))
}

fn build_pie_svg(c: &PieConfig, df: &DataFrame) -> Option<String> {
    let labels = extract_str(df, &c.label_col)?;
    let values = extract_f64(df, &c.value_col)?;
    if labels.is_empty() || values.is_empty() {
        return None;
    }
    let palette = resolve_palette(c.palette.as_ref(), labels.len().max(1));
    Some(svg_plot::render_pie(
        &labels,
        &values,
        c.inner_radius.unwrap_or(0.0),
        &palette,
        c.show_legend.unwrap_or(true),
    ))
}

fn build_box_plot_svg(
    c: &BoxPlotConfig,
    df: &DataFrame,
    frames: &HashMap<String, DataFrame>,
) -> Option<String> {
    let cats = extract_str(df, &c.category_col)?;
    let q1 = extract_f64(df, &c.q1_col)?;
    let q2 = extract_f64(df, &c.q2_col)?;
    let q3 = extract_f64(df, &c.q3_col)?;
    let lower = extract_f64(df, &c.lower_col)?;
    let upper = extract_f64(df, &c.upper_col)?;
    if cats.is_empty() {
        return None;
    }
    let palette = if c.palette.is_some() {
        resolve_palette(c.palette.as_ref(), cats.len().max(1))
    } else {
        let one = c
            .color
            .clone()
            .unwrap_or_else(|| PLOT_PALETTE[0].to_string());
        vec![one; cats.len().max(1)]
    };

    let outliers: Option<(Vec<String>, Vec<f64>)> = match (&c.outlier_source_key, &c.outlier_value_col) {
        (Some(key), Some(val_col)) => frames.get(key).and_then(|odf| {
            let cats = extract_str(odf, &c.category_col)?;
            let vals = extract_f64(odf, val_col)?;
            Some((cats, vals))
        }),
        _ => None,
    };

    Some(svg_plot::render_box_plot(
        &cats,
        &q1,
        &q2,
        &q3,
        &lower,
        &upper,
        &palette,
        c.alpha.unwrap_or(0.7),
        outliers.as_ref().map(|(c, v)| (c.as_slice(), v.as_slice())),
        &c.y_label,
    ))
}

fn build_density_svg(c: &DensityConfig, df: &DataFrame) -> Option<String> {
    let cats = extract_str(df, &c.category_col)?;
    let vals = extract_f64(df, &c.value_col)?;
    if cats.is_empty() || vals.is_empty() {
        return None;
    }
    // Distinct categories, preserving first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for c in &cats {
        if seen.insert(c.clone()) {
            order.push(c.clone());
        }
    }
    let palette = if c.palette.is_some() {
        resolve_palette(c.palette.as_ref(), order.len().max(1))
    } else {
        let one = c
            .color
            .clone()
            .unwrap_or_else(|| PLOT_PALETTE[0].to_string());
        vec![one; order.len().max(1)]
    };
    Some(svg_plot::render_density(
        &cats,
        &vals,
        &order,
        &palette,
        c.alpha.unwrap_or(0.65),
        c.point_threshold.unwrap_or(50) as usize,
        &c.y_label,
    ))
}

// ── SVG plot renderer ────────────────────────────────────────────────────────

mod svg_plot {
    use std::fmt::Write as _;

    /// Total SVG width in user units (≈ pixels at 1× zoom).
    const W: f64 = 720.0;
    /// Total SVG height in user units.
    const H: f64 = 380.0;
    const MARGIN_TOP: f64 = 16.0;
    const MARGIN_BOTTOM: f64 = 56.0;
    const MARGIN_LEFT: f64 = 64.0;
    const MARGIN_RIGHT_NO_LEGEND: f64 = 24.0;
    const MARGIN_RIGHT_LEGEND: f64 = 150.0;

    pub(super) struct Plot {
        pub series: Vec<Series>,
        pub x_label: String,
        pub y_label: String,
        /// When set, overrides the nice-tick algorithm for the X axis. Each
        /// entry is `(numeric position, label text)`. Used for categorical or
        /// datetime X axes where the X data is just an index into a label list.
        pub x_ticks: Option<Vec<(f64, String)>>,
    }

    pub(super) struct Series {
        pub label: Option<String>,
        pub color: String,
        pub points: Vec<(f64, f64)>,
        pub kind: SeriesKind,
    }

    pub(super) enum SeriesKind {
        Line { width: f64, marker_radius: f64 },
        Scatter { radius: f64, alpha: f64 },
        Bubble { radii: Vec<f64>, alpha: f64 },
    }

    pub(super) fn render(p: &Plot) -> String {
        let has_legend = p.series.iter().any(|s| s.label.is_some()) && p.series.len() > 1;
        let margin_right = if has_legend {
            MARGIN_RIGHT_LEGEND
        } else {
            MARGIN_RIGHT_NO_LEGEND
        };
        let inner_w = W - MARGIN_LEFT - margin_right;
        let inner_h = H - MARGIN_TOP - MARGIN_BOTTOM;

        let (x_min, x_max, y_min, y_max) = match domain(p) {
            Some(d) => d,
            None => return empty_svg(p),
        };

        let mx = |x: f64| MARGIN_LEFT + (x - x_min) / (x_max - x_min) * inner_w;
        let my = |y: f64| MARGIN_TOP + (1.0 - (y - y_min) / (y_max - y_min)) * inner_h;

        // Custom ticks override the nice-tick algorithm; values that fall
        // outside the data domain are dropped.
        let xticks_with_labels: Vec<(f64, String)> = match &p.x_ticks {
            Some(ticks) => ticks
                .iter()
                .filter(|(v, _)| *v >= x_min && *v <= x_max)
                .cloned()
                .collect(),
            None => nice_ticks(x_min, x_max, 6)
                .into_iter()
                .map(|v| (v, fmt_tick(v)))
                .collect(),
        };
        let xticks: Vec<f64> = xticks_with_labels.iter().map(|(v, _)| *v).collect();
        let yticks = nice_ticks(y_min, y_max, 5);

        let mut s = String::new();
        write!(
            s,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W:.0} {H:.0}\" \
             width=\"{W:.0}\" height=\"{H:.0}\" font-family=\"system-ui, -apple-system, \
             'Segoe UI', sans-serif\" font-size=\"11\">",
        )
        .unwrap();
        // Page background.
        write!(
            s,
            "<rect width=\"{W:.0}\" height=\"{H:.0}\" fill=\"#ffffff\"/>",
        )
        .unwrap();
        // Plot area.
        write!(
            s,
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" \
             fill=\"#fafafa\" stroke=\"#bdbdbd\" stroke-width=\"0.6\"/>",
            MARGIN_LEFT, MARGIN_TOP, inner_w, inner_h,
        )
        .unwrap();

        // Gridlines.
        for &x in &xticks {
            let px = mx(x);
            write!(
                s,
                "<line x1=\"{px:.2}\" y1=\"{:.2}\" x2=\"{px:.2}\" y2=\"{:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>",
                MARGIN_TOP,
                MARGIN_TOP + inner_h,
            )
            .unwrap();
        }
        for &y in &yticks {
            let py = my(y);
            write!(
                s,
                "<line x1=\"{:.2}\" y1=\"{py:.2}\" x2=\"{:.2}\" y2=\"{py:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>",
                MARGIN_LEFT,
                MARGIN_LEFT + inner_w,
            )
            .unwrap();
        }

        // Tick labels.
        for (x, label) in &xticks_with_labels {
            let px = mx(*x);
            write!(
                s,
                "<text x=\"{px:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#555\">{}</text>",
                MARGIN_TOP + inner_h + 14.0,
                escape_xml(label),
            )
            .unwrap();
        }
        for &y in &yticks {
            let py = my(y);
            write!(
                s,
                "<text x=\"{:.2}\" y=\"{py:.2}\" text-anchor=\"end\" \
                 dominant-baseline=\"middle\" fill=\"#555\">{}</text>",
                MARGIN_LEFT - 6.0,
                escape_xml(&fmt_tick(y)),
            )
            .unwrap();
        }

        // Axis labels.
        if !p.x_label.is_empty() {
            write!(
                s,
                "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" \
                 font-size=\"12\">{}</text>",
                MARGIN_LEFT + inner_w / 2.0,
                H - 14.0,
                escape_xml(&p.x_label),
            )
            .unwrap();
        }
        if !p.y_label.is_empty() {
            write!(
                s,
                "<text x=\"0\" y=\"0\" transform=\"translate({:.2}, {:.2}) rotate(-90)\" \
                 text-anchor=\"middle\" fill=\"#333\" font-size=\"12\">{}</text>",
                14.0,
                MARGIN_TOP + inner_h / 2.0,
                escape_xml(&p.y_label),
            )
            .unwrap();
        }

        // Series glyphs.
        for ser in &p.series {
            match &ser.kind {
                SeriesKind::Line {
                    width,
                    marker_radius,
                } => {
                    let mut path = String::new();
                    let mut started = false;
                    for (x, y) in &ser.points {
                        if !x.is_finite() || !y.is_finite() {
                            started = false;
                            continue;
                        }
                        let cmd = if started { "L" } else { "M" };
                        let _ = write!(path, "{cmd} {:.2} {:.2} ", mx(*x), my(*y));
                        started = true;
                    }
                    if !path.is_empty() {
                        write!(
                            s,
                            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" \
                             stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                            path.trim_end(),
                            ser.color,
                            width,
                        )
                        .unwrap();
                    }
                    if *marker_radius > 0.0 {
                        for (x, y) in &ser.points {
                            if !x.is_finite() || !y.is_finite() {
                                continue;
                            }
                            write!(
                                s,
                                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>",
                                mx(*x),
                                my(*y),
                                marker_radius,
                                ser.color,
                            )
                            .unwrap();
                        }
                    }
                }
                SeriesKind::Scatter { radius, alpha } => {
                    for (x, y) in &ser.points {
                        if !x.is_finite() || !y.is_finite() {
                            continue;
                        }
                        write!(
                            s,
                            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\" \
                             fill-opacity=\"{:.2}\"/>",
                            mx(*x),
                            my(*y),
                            radius,
                            ser.color,
                            alpha,
                        )
                        .unwrap();
                    }
                }
                SeriesKind::Bubble { radii, alpha } => {
                    for (i, (x, y)) in ser.points.iter().enumerate() {
                        if !x.is_finite() || !y.is_finite() {
                            continue;
                        }
                        let r = radii.get(i).copied().unwrap_or(6.0);
                        write!(
                            s,
                            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\" \
                             fill-opacity=\"{:.2}\" stroke=\"{}\" stroke-opacity=\"0.5\" \
                             stroke-width=\"0.6\"/>",
                            mx(*x),
                            my(*y),
                            r,
                            ser.color,
                            alpha,
                            ser.color,
                        )
                        .unwrap();
                    }
                }
            }
        }

        // Legend.
        if has_legend {
            let lx = MARGIN_LEFT + inner_w + 14.0;
            let mut ly = MARGIN_TOP + 12.0;
            for ser in &p.series {
                if let Some(label) = &ser.label {
                    write!(
                        s,
                        "<rect x=\"{lx:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" \
                         fill=\"{}\"/>",
                        ly - 8.0,
                        ser.color,
                    )
                    .unwrap();
                    write!(
                        s,
                        "<text x=\"{:.2}\" y=\"{ly:.2}\" fill=\"#333\">{}</text>",
                        lx + 14.0,
                        escape_xml(label),
                    )
                    .unwrap();
                    ly += 16.0;
                }
            }
        }

        s.push_str("</svg>");
        s
    }

    fn domain(p: &Plot) -> Option<(f64, f64, f64, f64)> {
        let mut xmin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        let mut ymin = f64::INFINITY;
        let mut ymax = f64::NEG_INFINITY;
        for s in &p.series {
            for (x, y) in &s.points {
                if x.is_finite() {
                    xmin = xmin.min(*x);
                    xmax = xmax.max(*x);
                }
                if y.is_finite() {
                    ymin = ymin.min(*y);
                    ymax = ymax.max(*y);
                }
            }
        }
        if !xmin.is_finite() || !ymin.is_finite() {
            return None;
        }
        if xmax == xmin {
            xmin -= 0.5;
            xmax += 0.5;
        } else {
            let xpad = (xmax - xmin) * 0.05;
            xmin -= xpad;
            xmax += xpad;
        }
        if ymax == ymin {
            ymin -= 0.5;
            ymax += 0.5;
        } else {
            let ypad = (ymax - ymin) * 0.08;
            ymin -= ypad;
            ymax += ypad;
        }
        Some((xmin, xmax, ymin, ymax))
    }

    fn empty_svg(_p: &Plot) -> String {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W:.0} {H:.0}\" \
             width=\"{W:.0}\" height=\"{H:.0}\"><rect width=\"{W:.0}\" height=\"{H:.0}\" \
             fill=\"#ffffff\"/><text x=\"{:.0}\" y=\"{:.0}\" text-anchor=\"middle\" \
             fill=\"#888\" font-family=\"sans-serif\" font-size=\"12\">no data</text></svg>",
            W / 2.0,
            H / 2.0,
        )
    }

    /// Pick "nice" tick locations using the standard 1/2/5 progression.
    fn nice_ticks(min: f64, max: f64, target: usize) -> Vec<f64> {
        if !(min.is_finite() && max.is_finite()) || min >= max {
            return vec![];
        }
        let range = max - min;
        let raw = range / target.max(1) as f64;
        let mag = 10f64.powf(raw.log10().floor());
        let frac = raw / mag;
        let nice = if frac < 1.5 {
            1.0
        } else if frac < 3.0 {
            2.0
        } else if frac < 7.0 {
            5.0
        } else {
            10.0
        };
        let step = nice * mag;
        if !step.is_finite() || step <= 0.0 {
            return vec![];
        }
        let start = (min / step).ceil() * step;
        let mut out = Vec::new();
        let mut t = start;
        let max_ticks = 50;
        while t <= max + step * 1e-9 && out.len() < max_ticks {
            out.push(t);
            t += step;
        }
        out
    }

    fn fmt_tick(v: f64) -> String {
        if v == 0.0 {
            return "0".into();
        }
        let abs = v.abs();
        if abs >= 1e5 || abs < 1e-3 {
            format!("{:.2e}", v)
        } else {
            let s = format!("{:.4}", v);
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        }
    }

    fn escape_xml(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&apos;"),
                _ => out.push(c),
            }
        }
        out
    }

    fn truncate(s: &str, max: usize) -> String {
        if s.chars().count() <= max {
            s.to_string()
        } else {
            let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
            out.push('…');
            out
        }
    }

    fn open_svg(s: &mut String) {
        write!(
            s,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W:.0} {H:.0}\" \
             width=\"{W:.0}\" height=\"{H:.0}\" font-family=\"system-ui, -apple-system, \
             'Segoe UI', sans-serif\" font-size=\"11\">\
             <rect width=\"{W:.0}\" height=\"{H:.0}\" fill=\"#ffffff\"/>",
        )
        .unwrap();
    }

    fn plot_rect(s: &mut String, left: f64, top: f64, w: f64, h: f64) {
        write!(
            s,
            "<rect x=\"{left:.2}\" y=\"{top:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" \
             fill=\"#fafafa\" stroke=\"#bdbdbd\" stroke-width=\"0.6\"/>",
        )
        .unwrap();
    }

    fn x_axis_label(s: &mut String, label: &str, left: f64, w: f64) {
        if !label.is_empty() {
            write!(
                s,
                "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" \
                 font-size=\"12\">{}</text>",
                left + w / 2.0,
                H - 14.0,
                escape_xml(label),
            )
            .unwrap();
        }
    }

    fn y_axis_label(s: &mut String, label: &str, top: f64, h: f64) {
        if !label.is_empty() {
            write!(
                s,
                "<text x=\"0\" y=\"0\" transform=\"translate(14.00, {:.2}) rotate(-90)\" \
                 text-anchor=\"middle\" fill=\"#333\" font-size=\"12\">{}</text>",
                top + h / 2.0,
                escape_xml(label),
            )
            .unwrap();
        }
    }

    /// Horizontal bar chart: categorical Y axis, numeric X axis.
    pub(super) fn render_hbar(
        cats: &[String],
        values: &[f64],
        color: &str,
        x_label: &str,
    ) -> String {
        let n = cats.len().min(values.len());
        if n == 0 {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: String::new(),
                x_ticks: None,
            });
        }
        let valid: Vec<f64> = values[..n]
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect();
        let mut max_v = valid.iter().copied().fold(0.0_f64, f64::max);
        let min_v = valid.iter().copied().fold(0.0_f64, f64::min);
        if max_v <= min_v {
            max_v = min_v + 1.0;
        }
        let pad = (max_v - min_v) * 0.05;
        let x_lo = if min_v >= 0.0 { 0.0 } else { min_v - pad };
        let x_hi = if max_v <= 0.0 { 0.0 } else { max_v + pad };

        let left = 130.0;
        let right = 24.0;
        let top = MARGIN_TOP;
        let bottom = MARGIN_BOTTOM;
        let inner_w = W - left - right;
        let inner_h = H - top - bottom;

        let mx = |x: f64| left + (x - x_lo) / (x_hi - x_lo) * inner_w;
        let row_h = inner_h / n as f64;
        let bar_h = row_h * 0.7;

        let mut s = String::new();
        open_svg(&mut s);
        plot_rect(&mut s, left, top, inner_w, inner_h);

        let xticks = nice_ticks(x_lo, x_hi, 5);
        for &x in &xticks {
            let px = mx(x);
            write!(
                s,
                "<line x1=\"{px:.2}\" y1=\"{top:.2}\" x2=\"{px:.2}\" y2=\"{:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>\
                 <text x=\"{px:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#555\">{}</text>",
                top + inner_h,
                top + inner_h + 14.0,
                escape_xml(&fmt_tick(x)),
            )
            .unwrap();
        }
        let zero_x = if x_lo <= 0.0 && 0.0 <= x_hi {
            mx(0.0)
        } else {
            mx(x_lo)
        };

        for (i, (cat, v)) in cats.iter().zip(values.iter()).enumerate().take(n) {
            let cy = top + (i as f64 + 0.5) * row_h;
            if v.is_finite() {
                let bx = mx(*v);
                let (rx, rw) = if bx >= zero_x {
                    (zero_x, bx - zero_x)
                } else {
                    (bx, zero_x - bx)
                };
                write!(
                    s,
                    "<rect x=\"{rx:.2}\" y=\"{:.2}\" width=\"{rw:.2}\" height=\"{bar_h:.2}\" \
                     fill=\"{color}\"/>",
                    cy - bar_h / 2.0,
                )
                .unwrap();
            }
            write!(
                s,
                "<text x=\"{:.2}\" y=\"{cy:.2}\" text-anchor=\"end\" \
                 dominant-baseline=\"middle\" fill=\"#333\">{}</text>",
                left - 6.0,
                escape_xml(&truncate(cat, 22)),
            )
            .unwrap();
        }

        x_axis_label(&mut s, x_label, left, inner_w);
        s.push_str("</svg>");
        s
    }

    /// Vertical clustered bar chart: categorical X axis, numeric Y axis,
    /// multiple bars per category coloured by group.
    pub(super) fn render_grouped_bar(
        x_cats: &[String],
        groups: &[String],
        values: &[f64],
        palette: &[String],
        y_label: &str,
        bar_width: f64,
    ) -> String {
        let n = x_cats.len().min(groups.len()).min(values.len());
        if n == 0 {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        // Distinct x and group values, preserving first-seen order.
        let mut x_order: Vec<String> = Vec::new();
        let mut g_order: Vec<String> = Vec::new();
        for i in 0..n {
            if !x_order.contains(&x_cats[i]) {
                x_order.push(x_cats[i].clone());
            }
            if !g_order.contains(&groups[i]) {
                g_order.push(groups[i].clone());
            }
        }
        // Build matrix (x_idx, g_idx) → value (last write wins if duplicates).
        let mut matrix = vec![vec![f64::NAN; g_order.len()]; x_order.len()];
        for i in 0..n {
            if let (Some(xi), Some(gi)) = (
                x_order.iter().position(|x| x == &x_cats[i]),
                g_order.iter().position(|g| g == &groups[i]),
            ) {
                matrix[xi][gi] = values[i];
            }
        }
        let max_v = matrix
            .iter()
            .flatten()
            .copied()
            .filter(|v| v.is_finite())
            .fold(0.0_f64, f64::max);
        let min_v = matrix
            .iter()
            .flatten()
            .copied()
            .filter(|v| v.is_finite())
            .fold(0.0_f64, f64::min);
        let pad = (max_v - min_v).abs().max(1.0) * 0.08;
        let y_lo = if min_v >= 0.0 { 0.0 } else { min_v - pad };
        let y_hi = if max_v <= 0.0 { 0.0 } else { max_v + pad };

        let has_legend = g_order.len() > 1;
        let left = MARGIN_LEFT;
        let right = if has_legend {
            MARGIN_RIGHT_LEGEND
        } else {
            MARGIN_RIGHT_NO_LEGEND
        };
        let top = MARGIN_TOP;
        let bottom = MARGIN_BOTTOM;
        let inner_w = W - left - right;
        let inner_h = H - top - bottom;

        let my = |y: f64| top + (1.0 - (y - y_lo) / (y_hi - y_lo)) * inner_h;
        let band_w = inner_w / x_order.len() as f64;
        let cluster_w = band_w * bar_width.clamp(0.1, 1.0);
        let bar_w = cluster_w / g_order.len() as f64;

        let mut s = String::new();
        open_svg(&mut s);
        plot_rect(&mut s, left, top, inner_w, inner_h);

        let yticks = nice_ticks(y_lo, y_hi, 5);
        for &y in &yticks {
            let py = my(y);
            write!(
                s,
                "<line x1=\"{:.2}\" y1=\"{py:.2}\" x2=\"{:.2}\" y2=\"{py:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>\
                 <text x=\"{:.2}\" y=\"{py:.2}\" text-anchor=\"end\" \
                 dominant-baseline=\"middle\" fill=\"#555\">{}</text>",
                left,
                left + inner_w,
                left - 6.0,
                escape_xml(&fmt_tick(y)),
            )
            .unwrap();
        }

        // X tick labels and bars.
        let zero_y = if y_lo <= 0.0 && 0.0 <= y_hi { my(0.0) } else { my(y_lo) };
        for (xi, x_cat) in x_order.iter().enumerate() {
            let band_cx = left + (xi as f64 + 0.5) * band_w;
            let cluster_x0 = band_cx - cluster_w / 2.0;
            for (gi, _g) in g_order.iter().enumerate() {
                let v = matrix[xi][gi];
                if !v.is_finite() {
                    continue;
                }
                let bx = cluster_x0 + gi as f64 * bar_w;
                let by = my(v);
                let (ry, rh) = if by <= zero_y {
                    (by, zero_y - by)
                } else {
                    (zero_y, by - zero_y)
                };
                write!(
                    s,
                    "<rect x=\"{bx:.2}\" y=\"{ry:.2}\" width=\"{:.2}\" height=\"{rh:.2}\" \
                     fill=\"{}\"/>",
                    bar_w * 0.92,
                    palette[gi % palette.len()],
                )
                .unwrap();
            }
            write!(
                s,
                "<text x=\"{band_cx:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#555\">{}</text>",
                top + inner_h + 14.0,
                escape_xml(&truncate(x_cat, 14)),
            )
            .unwrap();
        }

        if has_legend {
            let lx = left + inner_w + 14.0;
            let mut ly = top + 12.0;
            for (gi, g) in g_order.iter().enumerate() {
                write!(
                    s,
                    "<rect x=\"{lx:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" fill=\"{}\"/>\
                     <text x=\"{:.2}\" y=\"{ly:.2}\" fill=\"#333\">{}</text>",
                    ly - 8.0,
                    palette[gi % palette.len()],
                    lx + 14.0,
                    escape_xml(&truncate(g, 14)),
                )
                .unwrap();
                ly += 16.0;
            }
        }

        y_axis_label(&mut s, y_label, top, inner_h);
        s.push_str("</svg>");
        s
    }

    /// Histogram from pre-computed bin edges + values. CDF mode draws a step
    /// line; count/PDF modes draw filled bars per bin.
    pub(super) fn render_histogram(
        lefts: &[f64],
        rights: &[f64],
        values: &[f64],
        is_cdf: bool,
        color: &str,
        alpha: f64,
        x_label: &str,
        y_label: &str,
    ) -> String {
        let n = lefts.len().min(rights.len()).min(values.len());
        if n == 0 {
            return empty_svg(&Plot {
                series: vec![],
                x_label: x_label.into(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        let x_lo = lefts[..n]
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .fold(f64::INFINITY, f64::min);
        let x_hi = rights[..n]
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .fold(f64::NEG_INFINITY, f64::max);
        if !x_lo.is_finite() || !x_hi.is_finite() || x_hi <= x_lo {
            return empty_svg(&Plot {
                series: vec![],
                x_label: x_label.into(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        let max_v = values[..n]
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .fold(0.0_f64, f64::max);
        let y_hi = if is_cdf { 1.05 } else { max_v * 1.08 };
        let y_lo = 0.0;

        let left = MARGIN_LEFT;
        let right = MARGIN_RIGHT_NO_LEGEND;
        let top = MARGIN_TOP;
        let bottom = MARGIN_BOTTOM;
        let inner_w = W - left - right;
        let inner_h = H - top - bottom;

        let mx = |x: f64| left + (x - x_lo) / (x_hi - x_lo) * inner_w;
        let my = |y: f64| top + (1.0 - (y - y_lo) / (y_hi - y_lo)) * inner_h;

        let mut s = String::new();
        open_svg(&mut s);
        plot_rect(&mut s, left, top, inner_w, inner_h);

        let xticks = nice_ticks(x_lo, x_hi, 6);
        let yticks = nice_ticks(y_lo, y_hi, 5);
        for &x in &xticks {
            let px = mx(x);
            write!(
                s,
                "<line x1=\"{px:.2}\" y1=\"{top:.2}\" x2=\"{px:.2}\" y2=\"{:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>\
                 <text x=\"{px:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#555\">{}</text>",
                top + inner_h,
                top + inner_h + 14.0,
                escape_xml(&fmt_tick(x)),
            )
            .unwrap();
        }
        for &y in &yticks {
            let py = my(y);
            write!(
                s,
                "<line x1=\"{left:.2}\" y1=\"{py:.2}\" x2=\"{:.2}\" y2=\"{py:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>\
                 <text x=\"{:.2}\" y=\"{py:.2}\" text-anchor=\"end\" \
                 dominant-baseline=\"middle\" fill=\"#555\">{}</text>",
                left + inner_w,
                left - 6.0,
                escape_xml(&fmt_tick(y)),
            )
            .unwrap();
        }

        if is_cdf {
            let mut path = String::new();
            for i in 0..n {
                let l = lefts[i];
                let r = rights[i];
                let v = values[i];
                if !l.is_finite() || !r.is_finite() || !v.is_finite() {
                    continue;
                }
                let prev_v = if i == 0 { 0.0 } else { values[i - 1] };
                let pl = mx(l);
                let pr = mx(r);
                let py_v = my(v);
                let py_prev = my(prev_v);
                if path.is_empty() {
                    let _ = write!(path, "M {pl:.2} {py_prev:.2} ");
                }
                let _ = write!(path, "L {pl:.2} {py_v:.2} L {pr:.2} {py_v:.2} ");
            }
            if !path.is_empty() {
                write!(
                    s,
                    "<path d=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"1.8\" \
                     stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                    path.trim_end(),
                )
                .unwrap();
            }
        } else {
            for i in 0..n {
                let l = lefts[i];
                let r = rights[i];
                let v = values[i];
                if !l.is_finite() || !r.is_finite() || !v.is_finite() || v <= 0.0 {
                    continue;
                }
                let x0 = mx(l);
                let x1 = mx(r);
                let y0 = my(v);
                let y1 = my(0.0);
                write!(
                    s,
                    "<rect x=\"{:.2}\" y=\"{y0:.2}\" width=\"{:.2}\" height=\"{:.2}\" \
                     fill=\"{color}\" fill-opacity=\"{alpha:.2}\" stroke=\"#ffffff\" \
                     stroke-width=\"0.5\"/>",
                    x0,
                    (x1 - x0).max(0.5),
                    (y1 - y0).max(0.0),
                )
                .unwrap();
            }
        }

        x_axis_label(&mut s, x_label, left, inner_w);
        y_axis_label(&mut s, y_label, top, inner_h);
        s.push_str("</svg>");
        s
    }

    /// Pie or donut chart. `inner_radius` is a fraction (0.0–0.9) of the
    /// outer radius; `0.0` produces a solid pie.
    pub(super) fn render_pie(
        labels: &[String],
        values: &[f64],
        inner_radius_frac: f64,
        palette: &[String],
        show_legend: bool,
    ) -> String {
        let n = labels.len().min(values.len());
        let total: f64 = values[..n]
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .sum();
        if n == 0 || total <= 0.0 {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: String::new(),
                x_ticks: None,
            });
        }

        let left = 24.0;
        let right = if show_legend {
            MARGIN_RIGHT_LEGEND + 30.0
        } else {
            24.0
        };
        let top = MARGIN_TOP;
        let bottom = MARGIN_BOTTOM;
        let inner_w = W - left - right;
        let inner_h = H - top - bottom;

        let cx = left + inner_w / 2.0;
        let cy = top + inner_h / 2.0;
        let r_outer = inner_w.min(inner_h) / 2.0 - 6.0;
        let r_inner = r_outer * inner_radius_frac.clamp(0.0, 0.9);

        let mut s = String::new();
        open_svg(&mut s);

        let mut start = -std::f64::consts::FRAC_PI_2; // start at 12 o'clock
        for (i, (_label, &v)) in labels.iter().zip(values.iter()).enumerate().take(n) {
            if !v.is_finite() || v <= 0.0 {
                continue;
            }
            let frac = v / total;
            let angle = frac * std::f64::consts::TAU;
            let stop = start + angle;
            let large_arc = if angle > std::f64::consts::PI { 1 } else { 0 };
            let color = &palette[i % palette.len()];

            let x0 = cx + r_outer * start.cos();
            let y0 = cy + r_outer * start.sin();
            let x1 = cx + r_outer * stop.cos();
            let y1 = cy + r_outer * stop.sin();

            if r_inner <= 0.0 {
                write!(
                    s,
                    "<path d=\"M {cx:.2} {cy:.2} L {x0:.2} {y0:.2} \
                     A {r_outer:.2} {r_outer:.2} 0 {large_arc} 1 {x1:.2} {y1:.2} Z\" \
                     fill=\"{color}\" stroke=\"#ffffff\" stroke-width=\"1\"/>",
                )
                .unwrap();
            } else {
                let xi0 = cx + r_inner * start.cos();
                let yi0 = cy + r_inner * start.sin();
                let xi1 = cx + r_inner * stop.cos();
                let yi1 = cy + r_inner * stop.sin();
                write!(
                    s,
                    "<path d=\"M {x0:.2} {y0:.2} \
                     A {r_outer:.2} {r_outer:.2} 0 {large_arc} 1 {x1:.2} {y1:.2} \
                     L {xi1:.2} {yi1:.2} \
                     A {r_inner:.2} {r_inner:.2} 0 {large_arc} 0 {xi0:.2} {yi0:.2} Z\" \
                     fill=\"{color}\" stroke=\"#ffffff\" stroke-width=\"1\"/>",
                )
                .unwrap();
            }
            start = stop;
        }

        if show_legend {
            let lx = left + inner_w + 14.0;
            let mut ly = top + 12.0;
            for (i, label) in labels.iter().enumerate().take(n) {
                let v = values.get(i).copied().unwrap_or(0.0);
                if !v.is_finite() || v <= 0.0 {
                    continue;
                }
                let pct = v / total * 100.0;
                write!(
                    s,
                    "<rect x=\"{lx:.2}\" y=\"{:.2}\" width=\"10\" height=\"10\" fill=\"{}\"/>\
                     <text x=\"{:.2}\" y=\"{ly:.2}\" fill=\"#333\">{} ({:.1}%)</text>",
                    ly - 8.0,
                    palette[i % palette.len()],
                    lx + 14.0,
                    escape_xml(&truncate(label, 18)),
                    pct,
                )
                .unwrap();
                ly += 16.0;
                if ly > top + inner_h - 4.0 {
                    break;
                }
            }
        }

        s.push_str("</svg>");
        s
    }

    /// Box-and-whisker chart. Per-category boxes spanning q1..q3 with a
    /// median bar at q2 and whiskers from `lower` to `upper`. Optional
    /// outliers render as small dots.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_box_plot(
        cats: &[String],
        q1: &[f64],
        q2: &[f64],
        q3: &[f64],
        lower: &[f64],
        upper: &[f64],
        palette: &[String],
        alpha: f64,
        outliers: Option<(&[String], &[f64])>,
        y_label: &str,
    ) -> String {
        let n = cats
            .len()
            .min(q1.len())
            .min(q2.len())
            .min(q3.len())
            .min(lower.len())
            .min(upper.len());
        if n == 0 {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for i in 0..n {
            for v in [lower[i], upper[i], q1[i], q3[i]] {
                if v.is_finite() {
                    y_min = y_min.min(v);
                    y_max = y_max.max(v);
                }
            }
        }
        if let Some((_, ovals)) = outliers {
            for &v in ovals {
                if v.is_finite() {
                    y_min = y_min.min(v);
                    y_max = y_max.max(v);
                }
            }
        }
        if !y_min.is_finite() || !y_max.is_finite() {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        if y_max == y_min {
            y_max += 1.0;
            y_min -= 1.0;
        }
        let pad = (y_max - y_min) * 0.08;
        let y_lo = y_min - pad;
        let y_hi = y_max + pad;

        let left = MARGIN_LEFT;
        let right = MARGIN_RIGHT_NO_LEGEND;
        let top = MARGIN_TOP;
        let bottom = MARGIN_BOTTOM;
        let inner_w = W - left - right;
        let inner_h = H - top - bottom;

        let band_w = inner_w / n as f64;
        let box_w = band_w * 0.55;
        let my = |y: f64| top + (1.0 - (y - y_lo) / (y_hi - y_lo)) * inner_h;

        let mut s = String::new();
        open_svg(&mut s);
        plot_rect(&mut s, left, top, inner_w, inner_h);

        let yticks = nice_ticks(y_lo, y_hi, 6);
        for &y in &yticks {
            let py = my(y);
            write!(
                s,
                "<line x1=\"{left:.2}\" y1=\"{py:.2}\" x2=\"{:.2}\" y2=\"{py:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>\
                 <text x=\"{:.2}\" y=\"{py:.2}\" text-anchor=\"end\" \
                 dominant-baseline=\"middle\" fill=\"#555\">{}</text>",
                left + inner_w,
                left - 6.0,
                escape_xml(&fmt_tick(y)),
            )
            .unwrap();
        }

        for i in 0..n {
            let cx = left + (i as f64 + 0.5) * band_w;
            let bx = cx - box_w / 2.0;
            let color = &palette[i % palette.len()];
            if q1[i].is_finite() && q3[i].is_finite() {
                let py_q3 = my(q3[i]);
                let py_q1 = my(q1[i]);
                write!(
                    s,
                    "<rect x=\"{bx:.2}\" y=\"{py_q3:.2}\" width=\"{box_w:.2}\" \
                     height=\"{:.2}\" fill=\"{color}\" fill-opacity=\"{alpha:.2}\" \
                     stroke=\"{color}\" stroke-width=\"1\"/>",
                    (py_q1 - py_q3).max(0.0),
                )
                .unwrap();
            }
            if q2[i].is_finite() {
                let py_med = my(q2[i]);
                write!(
                    s,
                    "<line x1=\"{bx:.2}\" y1=\"{py_med:.2}\" x2=\"{:.2}\" y2=\"{py_med:.2}\" \
                     stroke=\"#222\" stroke-width=\"1.6\"/>",
                    bx + box_w,
                )
                .unwrap();
            }
            // Whiskers + caps.
            if lower[i].is_finite() && q1[i].is_finite() {
                let py_lo = my(lower[i]);
                let py_q1 = my(q1[i]);
                write!(
                    s,
                    "<line x1=\"{cx:.2}\" y1=\"{py_q1:.2}\" x2=\"{cx:.2}\" y2=\"{py_lo:.2}\" \
                     stroke=\"{color}\" stroke-width=\"1\"/>\
                     <line x1=\"{:.2}\" y1=\"{py_lo:.2}\" x2=\"{:.2}\" y2=\"{py_lo:.2}\" \
                     stroke=\"{color}\" stroke-width=\"1\"/>",
                    cx - box_w / 4.0,
                    cx + box_w / 4.0,
                )
                .unwrap();
            }
            if upper[i].is_finite() && q3[i].is_finite() {
                let py_hi = my(upper[i]);
                let py_q3 = my(q3[i]);
                write!(
                    s,
                    "<line x1=\"{cx:.2}\" y1=\"{py_q3:.2}\" x2=\"{cx:.2}\" y2=\"{py_hi:.2}\" \
                     stroke=\"{color}\" stroke-width=\"1\"/>\
                     <line x1=\"{:.2}\" y1=\"{py_hi:.2}\" x2=\"{:.2}\" y2=\"{py_hi:.2}\" \
                     stroke=\"{color}\" stroke-width=\"1\"/>",
                    cx - box_w / 4.0,
                    cx + box_w / 4.0,
                )
                .unwrap();
            }
            // X tick label.
            write!(
                s,
                "<text x=\"{cx:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#555\">{}</text>",
                top + inner_h + 14.0,
                escape_xml(&truncate(&cats[i], 14)),
            )
            .unwrap();
        }

        if let Some((ocats, ovals)) = outliers {
            for (cat, &v) in ocats.iter().zip(ovals.iter()) {
                if !v.is_finite() {
                    continue;
                }
                let Some(idx) = cats.iter().position(|c| c == cat) else {
                    continue;
                };
                let cx = left + (idx as f64 + 0.5) * band_w;
                let py = my(v);
                let color = &palette[idx % palette.len()];
                write!(
                    s,
                    "<circle cx=\"{cx:.2}\" cy=\"{py:.2}\" r=\"2.5\" fill=\"{color}\" \
                     fill-opacity=\"0.7\"/>",
                )
                .unwrap();
            }
        }

        y_axis_label(&mut s, y_label, top, inner_h);
        s.push_str("</svg>");
        s
    }

    /// Density plot: violin (KDE polygon) per category when point count exceeds
    /// `point_threshold`, otherwise sina (jittered scatter). `cats`/`vals` are
    /// long-format observations; `order` lists distinct categories in the
    /// order they should appear on the X axis.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_density(
        cats: &[String],
        vals: &[f64],
        order: &[String],
        palette: &[String],
        alpha: f64,
        point_threshold: usize,
        y_label: &str,
    ) -> String {
        let n = cats.len().min(vals.len());
        if n == 0 || order.is_empty() {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        // Group values by category.
        let mut groups: Vec<Vec<f64>> = vec![Vec::new(); order.len()];
        for i in 0..n {
            if !vals[i].is_finite() {
                continue;
            }
            if let Some(idx) = order.iter().position(|c| c == &cats[i]) {
                groups[idx].push(vals[i]);
            }
        }
        let all: Vec<f64> = groups.iter().flatten().copied().collect();
        if all.is_empty() {
            return empty_svg(&Plot {
                series: vec![],
                x_label: String::new(),
                y_label: y_label.into(),
                x_ticks: None,
            });
        }
        let y_min = all.iter().copied().fold(f64::INFINITY, f64::min);
        let y_max = all.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let pad = (y_max - y_min).abs().max(1.0) * 0.08;
        let y_lo = y_min - pad;
        let y_hi = y_max + pad;

        let left = MARGIN_LEFT;
        let right = MARGIN_RIGHT_NO_LEGEND;
        let top = MARGIN_TOP;
        let bottom = MARGIN_BOTTOM;
        let inner_w = W - left - right;
        let inner_h = H - top - bottom;
        let band_w = inner_w / order.len() as f64;
        let half_w = band_w * 0.42;
        let my = |y: f64| top + (1.0 - (y - y_lo) / (y_hi - y_lo)) * inner_h;

        let mut s = String::new();
        open_svg(&mut s);
        plot_rect(&mut s, left, top, inner_w, inner_h);

        let yticks = nice_ticks(y_lo, y_hi, 6);
        for &y in &yticks {
            let py = my(y);
            write!(
                s,
                "<line x1=\"{left:.2}\" y1=\"{py:.2}\" x2=\"{:.2}\" y2=\"{py:.2}\" \
                 stroke=\"#e6e6e6\" stroke-width=\"0.5\"/>\
                 <text x=\"{:.2}\" y=\"{py:.2}\" text-anchor=\"end\" \
                 dominant-baseline=\"middle\" fill=\"#555\">{}</text>",
                left + inner_w,
                left - 6.0,
                escape_xml(&fmt_tick(y)),
            )
            .unwrap();
        }

        // Use a small deterministic PRNG for jitter so output is stable.
        let mut rng_state: u64 = 0xC0FFEE;
        let next_uniform = |state: &mut u64| {
            *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((*state >> 33) as f64) / (u32::MAX as f64)
        };

        for (i, vals_i) in groups.iter().enumerate() {
            let cx = left + (i as f64 + 0.5) * band_w;
            let color = &palette[i % palette.len()];
            if vals_i.is_empty() {
                // category tick label only
            } else if vals_i.len() > point_threshold {
                // Violin: KDE polygon mirrored around cx.
                let grid = linspace(y_lo, y_hi, 64);
                let kde = gaussian_kde(vals_i, &grid);
                let max_k = kde.iter().copied().fold(0.0_f64, f64::max).max(1e-12);
                let mut path = String::from("M ");
                for (j, &g) in grid.iter().enumerate() {
                    let k = kde[j] / max_k * half_w;
                    let _ = write!(path, "{:.2} {:.2} ", cx + k, my(g));
                    if j < grid.len() - 1 {
                        path.push_str("L ");
                    }
                }
                for (j, &g) in grid.iter().enumerate().rev() {
                    let k = kde[j] / max_k * half_w;
                    path.push_str("L ");
                    let _ = write!(path, "{:.2} {:.2} ", cx - k, my(g));
                }
                path.push('Z');
                write!(
                    s,
                    "<path d=\"{path}\" fill=\"{color}\" fill-opacity=\"{alpha:.2}\" \
                     stroke=\"{color}\" stroke-width=\"1\"/>",
                )
                .unwrap();
                // Median line.
                let med = median(vals_i);
                if med.is_finite() {
                    let py = my(med);
                    write!(
                        s,
                        "<line x1=\"{:.2}\" y1=\"{py:.2}\" x2=\"{:.2}\" y2=\"{py:.2}\" \
                         stroke=\"#222\" stroke-width=\"1.4\"/>",
                        cx - half_w * 0.6,
                        cx + half_w * 0.6,
                    )
                    .unwrap();
                }
            } else {
                // Sina: jittered scatter using KDE width as envelope.
                let grid = linspace(y_lo, y_hi, 48);
                let kde = gaussian_kde(vals_i, &grid);
                let max_k = kde.iter().copied().fold(0.0_f64, f64::max).max(1e-12);
                for &v in vals_i {
                    let k = interp_kde(&kde, &grid, v) / max_k;
                    let env = k * half_w;
                    let u = next_uniform(&mut rng_state);
                    let dx = (u * 2.0 - 1.0) * env;
                    write!(
                        s,
                        "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"3\" fill=\"{color}\" \
                         fill-opacity=\"{alpha:.2}\"/>",
                        cx + dx,
                        my(v),
                    )
                    .unwrap();
                }
            }
            write!(
                s,
                "<text x=\"{cx:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#555\">{}</text>",
                top + inner_h + 14.0,
                escape_xml(&truncate(&order[i], 14)),
            )
            .unwrap();
        }

        y_axis_label(&mut s, y_label, top, inner_h);
        s.push_str("</svg>");
        s
    }

    fn linspace(lo: f64, hi: f64, n: usize) -> Vec<f64> {
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![lo];
        }
        let step = (hi - lo) / (n - 1) as f64;
        (0..n).map(|i| lo + i as f64 * step).collect()
    }

    fn gaussian_kde(values: &[f64], grid: &[f64]) -> Vec<f64> {
        if values.is_empty() {
            return vec![0.0; grid.len()];
        }
        let n = values.len() as f64;
        let mean = values.iter().copied().sum::<f64>() / n;
        let var = if values.len() > 1 {
            values.iter().map(|&v| (v - mean) * (v - mean)).sum::<f64>() / (n - 1.0)
        } else {
            1.0
        };
        let std = var.sqrt().max(1e-10);
        let h = (1.06 * std * n.powf(-0.2)).max(1e-6);
        let norm = 1.0 / (n * h * (2.0 * std::f64::consts::PI).sqrt());
        grid.iter()
            .map(|&y| {
                norm * values
                    .iter()
                    .map(|&v| {
                        let z = (y - v) / h;
                        (-0.5 * z * z).exp()
                    })
                    .sum::<f64>()
            })
            .collect()
    }

    fn interp_kde(kde: &[f64], grid: &[f64], y: f64) -> f64 {
        if grid.is_empty() {
            return 0.0;
        }
        let n = grid.len();
        if y <= grid[0] {
            return kde[0];
        }
        if y >= grid[n - 1] {
            return kde[n - 1];
        }
        let idx = grid.partition_point(|&g| g < y).saturating_sub(1);
        let i1 = (idx + 1).min(n - 1);
        let t = (y - grid[idx]) / (grid[i1] - grid[idx]).max(1e-12);
        kde[idx] * (1.0 - t) + kde[i1] * t
    }

    fn median(values: &[f64]) -> f64 {
        if values.is_empty() {
            return f64::NAN;
        }
        let mut sorted: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();
        if sorted.is_empty() {
            return f64::NAN;
        }
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len();
        if n % 2 == 1 {
            sorted[n / 2]
        } else {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nice_ticks_uses_one_two_five_progression() {
            let t = nice_ticks(0.0, 10.0, 5);
            assert_eq!(t, vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        }

        #[test]
        fn nice_ticks_handles_inverted_or_invalid() {
            assert!(nice_ticks(5.0, 1.0, 5).is_empty());
            assert!(nice_ticks(f64::NAN, 1.0, 5).is_empty());
        }

        #[test]
        fn render_emits_svg_root() {
            let p = Plot {
                series: vec![Series {
                    label: None,
                    color: "#4C72B0".into(),
                    points: vec![(0.0, 0.0), (1.0, 1.0)],
                    kind: SeriesKind::Scatter {
                        radius: 4.0,
                        alpha: 1.0,
                    },
                }],
                x_label: "X".into(),
                y_label: "Y".into(),
                x_ticks: None,
            };
            let s = render(&p);
            assert!(s.starts_with("<svg"));
            assert!(s.ends_with("</svg>"));
            assert!(s.contains("<circle"));
        }

        #[test]
        fn render_with_no_data_yields_placeholder() {
            let p = Plot {
                series: vec![Series {
                    label: None,
                    color: "#4C72B0".into(),
                    points: vec![],
                    kind: SeriesKind::Scatter {
                        radius: 4.0,
                        alpha: 1.0,
                    },
                }],
                x_label: String::new(),
                y_label: String::new(),
                x_ticks: None,
            };
            let s = render(&p);
            assert!(s.contains("no data"));
        }

        #[test]
        fn fmt_tick_trims_trailing_zeros() {
            assert_eq!(fmt_tick(1.0), "1");
            assert_eq!(fmt_tick(1.25), "1.25");
            assert_eq!(fmt_tick(0.0), "0");
        }
    }
}

// ── Data helpers ─────────────────────────────────────────────────────────────

/// Extract a column as `Vec<String>`. Tries the native `Utf8` accessor first,
/// then falls back to formatted `AnyValue` representation (with surrounding
/// quotes stripped) so categorical and other dtypes work too.
fn extract_str(df: &DataFrame, col: &str) -> Option<Vec<String>> {
    let s = df.column(col).ok()?;
    if let Ok(ca) = s.str() {
        return Some(
            ca.into_iter()
                .map(|o| o.unwrap_or("").to_string())
                .collect(),
        );
    }
    let mut out = Vec::with_capacity(s.len());
    for i in 0..s.len() {
        let av = s.get(i).ok()?;
        let mut formatted = format!("{av}");
        if formatted.starts_with('"') && formatted.ends_with('"') && formatted.len() >= 2 {
            formatted = formatted[1..formatted.len() - 1].to_string();
        }
        out.push(formatted);
    }
    Some(out)
}

/// Resolve a [`PaletteSpec`] into `n` concrete hex color strings, cycling as
/// needed. `Named` palettes fall back to the built-in [`PLOT_PALETTE`] since
/// this exporter does not embed Bokeh's named palette tables.
fn resolve_palette(spec: Option<&PaletteSpec>, n: usize) -> Vec<String> {
    let source: Vec<String> = match spec {
        Some(PaletteSpec::Custom(colors)) if !colors.is_empty() => colors.clone(),
        _ => PLOT_PALETTE.iter().map(|s| s.to_string()).collect(),
    };
    (0..n).map(|i| source[i % source.len()].clone()).collect()
}

fn extract_f64(df: &DataFrame, col: &str) -> Option<Vec<f64>> {
    let s = df.column(col).ok()?;
    let out: Vec<f64> = match s.dtype() {
        DataType::Float64 => s
            .f64()
            .ok()?
            .into_iter()
            .map(|o| o.unwrap_or(f64::NAN))
            .collect(),
        DataType::Float32 => s
            .f32()
            .ok()?
            .into_iter()
            .map(|o| o.map(f64::from).unwrap_or(f64::NAN))
            .collect(),
        DataType::Int64 => s
            .i64()
            .ok()?
            .into_iter()
            .map(|o| o.map(|v| v as f64).unwrap_or(f64::NAN))
            .collect(),
        DataType::Int32 => s
            .i32()
            .ok()?
            .into_iter()
            .map(|o| o.map(f64::from).unwrap_or(f64::NAN))
            .collect(),
        DataType::UInt32 => s
            .u32()
            .ok()?
            .into_iter()
            .map(|o| o.map(f64::from).unwrap_or(f64::NAN))
            .collect(),
        DataType::UInt64 => s
            .u64()
            .ok()?
            .into_iter()
            .map(|o| o.map(|v| v as f64).unwrap_or(f64::NAN))
            .collect(),
        _ => return None,
    };
    Some(out)
}

// ── Summary table ────────────────────────────────────────────────────────────

fn chart_summary(spec: &ChartSpec, df: &DataFrame) -> String {
    let cols: Vec<String> = match &spec.config {
        ChartConfig::Line(c) => c.y_cols.clone(),
        ChartConfig::Scatter(c) => vec![c.y_col.clone()],
        ChartConfig::Bubble(c) => vec![c.y_col.clone(), c.size_col.clone()],
        ChartConfig::HBar(c) => vec![c.value_col.clone()],
        ChartConfig::GroupedBar(c) => vec![c.value_col.clone()],
        ChartConfig::Pie(c) => vec![c.value_col.clone()],
        ChartConfig::Histogram(c) => match c.display.as_ref().unwrap_or(&HistogramDisplay::Count) {
            HistogramDisplay::Count => vec!["count".to_string()],
            HistogramDisplay::Pdf => vec!["pdf".to_string()],
            HistogramDisplay::Cdf => vec!["cdf".to_string()],
        },
        ChartConfig::BoxPlot(_) | ChartConfig::Density(_) => return String::new(),
    };

    let rows: Vec<(String, f64, f64, f64)> = cols
        .into_iter()
        .filter_map(|name| numeric_summary(df, &name).map(|(mn, av, mx)| (name, mn, av, mx)))
        .collect();
    if rows.is_empty() {
        return String::new();
    }

    let mut s = String::new();
    s.push_str("#table(\n");
    s.push_str("  columns: (auto, 1fr, 1fr, 1fr,),\n");
    s.push_str("  align: (left, right, right, right,),\n");
    s.push_str("  table.header([*Series*], [*Min*], [*Mean*], [*Max*]),\n");
    for (name, mn, av, mx) in rows {
        let _ = writeln!(
            s,
            "  [{}], [{:.4}], [{:.4}], [{:.4}],",
            esc_markup(&name),
            mn,
            av,
            mx,
        );
    }
    s.push(')');
    s
}

fn numeric_summary(df: &DataFrame, col: &str) -> Option<(f64, f64, f64)> {
    let series = df.column(col).ok()?;
    let f: Vec<f64> = match series.dtype() {
        DataType::Float64 => series.f64().ok()?.into_iter().flatten().collect(),
        DataType::Float32 => series
            .f32()
            .ok()?
            .into_iter()
            .flatten()
            .map(f64::from)
            .collect(),
        DataType::Int64 => series
            .i64()
            .ok()?
            .into_iter()
            .flatten()
            .map(|v| v as f64)
            .collect(),
        DataType::Int32 => series
            .i32()
            .ok()?
            .into_iter()
            .flatten()
            .map(f64::from)
            .collect(),
        DataType::UInt32 => series
            .u32()
            .ok()?
            .into_iter()
            .flatten()
            .map(f64::from)
            .collect(),
        DataType::UInt64 => series
            .u64()
            .ok()?
            .into_iter()
            .flatten()
            .map(|v| v as f64)
            .collect(),
        _ => return None,
    };
    if f.is_empty() {
        return None;
    }
    let min = f.iter().copied().fold(f64::INFINITY, f64::min);
    let max = f.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = f.iter().sum::<f64>() / f.len() as f64;
    Some((min, mean, max))
}

fn format_cell(series: &polars::prelude::Column, row: usize, fmt: &ColumnFormat) -> String {
    let raw_val: Option<f64> = match series.dtype() {
        DataType::Float32 => series.f32().ok().and_then(|s| s.get(row)).map(f64::from),
        DataType::Float64 => series.f64().ok().and_then(|s| s.get(row)),
        DataType::Int32 => series.i32().ok().and_then(|s| s.get(row)).map(f64::from),
        DataType::Int64 => series.i64().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        DataType::UInt32 => series.u32().ok().and_then(|s| s.get(row)).map(f64::from),
        DataType::UInt64 => series.u64().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        _ => None,
    };
    if raw_val.is_none() {
        if let Ok(ca) = series.str() {
            return ca.get(row).unwrap_or("").to_string();
        }
        return series.get(row).map(|v| format!("{v}")).unwrap_or_default();
    }
    let v = raw_val.unwrap_or(0.0);
    match fmt {
        ColumnFormat::Text => format!("{v}"),
        ColumnFormat::Number { decimals } => {
            format!("{:.prec$}", v, prec = *decimals as usize)
        }
        ColumnFormat::Currency { symbol, decimals } => {
            let abs = v.abs();
            let sign = if v < 0.0 { "-" } else { "" };
            format!("{sign}{symbol}{:.prec$}", abs, prec = *decimals as usize)
        }
        ColumnFormat::Percent { decimals } => {
            format!("{:.prec$}%", v, prec = *decimals as usize)
        }
    }
}

/// Escape a string for safe inclusion in Typst markup (between `[ ]` content
/// blocks). Conservatively prefixes `\` to every character that has syntactic
/// meaning in markup mode, so plain text never collides with parser rules.
fn esc_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | '#' | '[' | ']' | '<' | '>' | '@' | '*' | '_' | '`' | '$' | '~' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Escape a string for safe inclusion inside a Typst double-quoted string
/// literal.
fn esc_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{ChartSpecBuilder, HBarConfig, LineConfig, ScatterConfig};
    use crate::handle::DfHandle;
    use crate::modules::{ParagraphSpec, StatGridSpec, StatItem, TableColumn, TableSpec};
    use crate::pages::PageBuilder;

    fn empty_frames() -> HashMap<String, DataFrame> {
        HashMap::new()
    }

    #[test]
    fn esc_markup_escapes_special_chars() {
        assert_eq!(esc_markup("a#b[c]"), "a\\#b\\[c\\]");
        assert_eq!(esc_markup("plain"), "plain");
        assert_eq!(esc_markup("a*b_c"), "a\\*b\\_c");
    }

    #[test]
    fn esc_str_escapes_quotes_and_backslashes() {
        assert_eq!(esc_str("hi \"there\""), "hi \\\"there\\\"");
        assert_eq!(esc_str("a\\b"), "a\\\\b");
        assert_eq!(esc_str("line\nbreak"), "line\\nbreak");
    }

    #[test]
    fn sanitize_slug_lowercases_and_replaces_punctuation() {
        assert_eq!(sanitize_slug("Hello, World!"), "hello_world");
        assert_eq!(sanitize_slug("  foo--bar  "), "foo_bar");
        assert_eq!(sanitize_slug(""), "");
    }

    #[test]
    fn empty_dashboard_emits_preamble_and_outline() {
        let report = build_typst_report(&[], &empty_frames(), "My Report");
        assert!(report.source.contains("#set page("));
        assert!(report.source.contains("My Report"));
        assert!(report.source.contains("#outline"));
        // Design tokens are emitted up front.
        assert!(report.source.contains("#let space ="));
        assert!(report.source.contains("#let accent ="));
        assert!(report.resources.is_empty());
    }

    #[test]
    fn page_renders_heading() {
        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let page = PageBuilder::new("p1", "Page One", "P1", 1)
            .chart(
                ChartSpecBuilder::hbar("Chart A", &DfHandle::new("d"), cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &empty_frames(), "");
        assert!(report.source.contains("plain-page(\"Page One\")"));
        assert!(report.source.contains("Chart A"));
        // Page slug becomes a Typst label so cross-refs can resolve.
        assert!(report.source.contains("<page-p1>"));
    }

    #[test]
    fn paragraph_lowers_with_title_and_body() {
        let para = ParagraphSpec::new("First.\n\nSecond.")
            .title("Intro")
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1)
            .paragraph(para)
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &empty_frames(), "");
        assert!(report.source.contains("== Intro"));
        assert!(report.source.contains("First."));
        assert!(report.source.contains("Second."));
    }

    #[test]
    fn stat_grid_lowers_to_typst_grid() {
        let grid = StatGridSpec::new()
            .item(StatItem::new("DURATION", "72").suffix("h"))
            .item(StatItem::new("PEAK", "4.2"))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1)
            .stat_grid(grid)
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &empty_frames(), "");
        assert!(report.source.contains("#grid("));
        assert!(report.source.contains("DURATION"));
        assert!(report.source.contains("PEAK"));
        assert!(report.source.contains("\"72\""));
    }

    #[test]
    fn table_lowers_with_header_and_rows() {
        let df = df![
            "name" => ["Alice", "Bob"],
            "score" => [95.0f64, 87.5],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("data".to_string(), df);

        let tbl = TableSpec::new("Scores", &DfHandle::new("data"))
            .column(TableColumn::text("name", "Name"))
            .column(TableColumn::number("score", "Score", 1))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1)
            .table(tbl)
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &frames, "");
        assert!(report.source.contains("== Scores"));
        assert!(report.source.contains("#table("));
        assert!(report.source.contains("Alice"));
        assert!(report.source.contains("95.0"));
    }

    fn line_report() -> TypstReport {
        let df = df![
            "x" => [1i64, 2, 3, 4],
            "y" => [10.0f64, 20.0, 30.0, 40.0],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("d".to_string(), df);

        let cfg = LineConfig::builder()
            .x("x")
            .y_cols(&["y"])
            .y_label("Y")
            .build()
            .unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(
                ChartSpecBuilder::line("Trend", &DfHandle::new("d"), cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();
        build_typst_report(&[page], &frames, "")
    }

    #[test]
    fn line_chart_emits_image_ref_and_svg_resource() {
        let report = line_report();
        assert!(report.source.contains("Trend"));
        assert!(report.source.contains("#image(\"resources/chart_001"));
        assert_eq!(report.resources.len(), 1);
        let res = &report.resources[0];
        assert!(res.relative_path.starts_with("resources/chart_001"));
        assert!(res.relative_path.ends_with(".svg"));
        let svg = std::str::from_utf8(&res.bytes).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("<path "));
    }

    #[test]
    fn line_chart_with_string_x_emits_svg() {
        let df = df![
            "month" => ["Jan", "Feb", "Mar", "Apr"],
            "revenue" => [100.0f64, 120.0, 150.0, 130.0],
            "profit" => [20.0f64, 25.0, 35.0, 28.0],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("d".to_string(), df);

        let cfg = LineConfig::builder()
            .x("month")
            .y_cols(&["revenue", "profit"])
            .y_label("USD")
            .build()
            .unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(
                ChartSpecBuilder::line("Trends", &DfHandle::new("d"), cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &frames, "");
        assert_eq!(report.resources.len(), 1);
        let svg = std::str::from_utf8(&report.resources[0].bytes).unwrap();
        // Two series → two `<path` elements for the lines.
        assert_eq!(svg.matches("<path ").count(), 2);
        // Tick labels carry month names instead of numeric indices.
        assert!(svg.contains(">Jan<"));
        assert!(svg.contains(">Apr<"));
    }

    #[test]
    fn line_chart_summary_still_present() {
        let report = line_report();
        assert!(report.source.contains("Min"));
        assert!(report.source.contains("Mean"));
        assert!(report.source.contains("Max"));
        assert!(report.source.contains("10.0000"));
        assert!(report.source.contains("40.0000"));
    }

    #[test]
    fn scatter_chart_emits_svg_with_circles() {
        let df = df![
            "x" => [1.0f64, 2.0],
            "y" => [5.0f64, 6.0],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("d".to_string(), df);

        let cfg = ScatterConfig::builder()
            .x("x")
            .y("y")
            .x_label("X")
            .y_label("Y")
            .build()
            .unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(
                ChartSpecBuilder::scatter("Pts", &DfHandle::new("d"), cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &frames, "");
        assert_eq!(report.resources.len(), 1);
        let svg = std::str::from_utf8(&report.resources[0].bytes).unwrap();
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn hbar_chart_emits_svg_with_bars() {
        let df = df![
            "c" => ["A", "B"],
            "v" => [1.0f64, 2.0],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("d".to_string(), df);

        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(
                ChartSpecBuilder::hbar("Bars", &DfHandle::new("d"), cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &frames, "");
        assert_eq!(report.resources.len(), 1);
        let svg = std::str::from_utf8(&report.resources[0].bytes).unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn source_inlined_replaces_image_refs_with_inline_bytes() {
        let report = line_report();
        let inlined = report.source_inlined();
        assert!(!inlined.contains("#image(\"resources/"));
        assert!(inlined.contains("#image(bytes("));
        assert!(inlined.contains("format: \"svg\""));
        // Original SVG payload (start tag) round-trips through Typst escaping.
        assert!(inlined.contains("<svg"));
    }

    #[test]
    fn write_to_creates_typ_and_resources_files() {
        let report = line_report();
        let dir = std::env::temp_dir().join(format!(
            "rust-to-bokeh-typst-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        report.write_to(&dir).unwrap();
        assert!(dir.join("report.typ").exists());
        let res_path = dir.join(&report.resources[0].relative_path);
        assert!(res_path.exists());
        let on_disk = std::fs::read(&res_path).unwrap();
        assert_eq!(on_disk, report.resources[0].bytes);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn category_summary_uses_summary_page_template() {
        let page = PageBuilder::new("temp", "Temperature — Cross-Test Summary", "Temp", 1)
            .category("Summary")
            .build()
            .unwrap();
        let report = build_typst_report(&[page], &empty_frames(), "");
        assert!(report
            .source
            .contains("summary-page(\"Temperature — Cross-Test Summary\")"));
        assert!(report.source.contains("<page-temp>"));
    }

    #[test]
    fn category_tests_uses_test_page_template() {
        let page = PageBuilder::new("test-overtemp", "Overtemp", "Overtemp", 1)
            .category("Tests")
            .build()
            .unwrap();
        let report = build_typst_report(&[page], &empty_frames(), "");
        assert!(report.source.contains("test-page(\"Overtemp\")"));
        assert!(report.source.contains("<page-test-overtemp>"));
    }

    #[test]
    fn anchor_cell_becomes_internal_link() {
        let df = df![
            "name" => [r#"<a href="test-overtemp.html">Overtemp</a>"#, "Plain"],
            "score" => [42i64, 7],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("d".to_string(), df);

        let tbl = TableSpec::new("Tests", &DfHandle::new("d"))
            .column(TableColumn::text("name", "Name"))
            .column(TableColumn::number("score", "Score", 0))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1).table(tbl).build().unwrap();
        let report = build_typst_report(&[page], &frames, "");
        assert!(report
            .source
            .contains("#link(<page-test-overtemp>)[Overtemp]"));
        // Plain cells are still escaped, no spurious link emitted.
        assert!(report.source.contains("[Plain]"));
    }

    #[test]
    fn anchor_with_external_url_falls_back_to_external_link() {
        let df = df![
            "url" => [r#"<a href="https://example.com">Docs</a>"#],
        ]
        .unwrap();
        let mut frames = HashMap::new();
        frames.insert("d".to_string(), df);

        let tbl = TableSpec::new("Refs", &DfHandle::new("d"))
            .column(TableColumn::text("url", "Link"))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1).table(tbl).build().unwrap();
        let report = build_typst_report(&[page], &frames, "");
        assert!(report
            .source
            .contains("#link(\"https://example.com\")[Docs]"));
    }

    #[test]
    fn missing_table_source_produces_warning_block() {
        let tbl = TableSpec::new("Missing", &DfHandle::new("nonexistent"))
            .column(TableColumn::text("x", "X"))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1)
            .table(tbl)
            .build()
            .unwrap();

        let report = build_typst_report(&[page], &empty_frames(), "");
        assert!(report.source.contains("Missing data for table source"));
    }

    #[test]
    fn pages_separated_by_pagebreak() {
        let mk_page = |slug: &str, title: &str| {
            let cfg = HBarConfig::builder()
                .category("c")
                .value("v")
                .x_label("X")
                .build()
                .unwrap();
            PageBuilder::new(slug, title, slug, 1)
                .chart(
                    ChartSpecBuilder::hbar("c", &DfHandle::new("d"), cfg)
                        .at(0, 0, 1)
                        .build(),
                )
                .build()
                .unwrap()
        };

        let report = build_typst_report(
            &[mk_page("a", "Alpha"), mk_page("b", "Beta")],
            &empty_frames(),
            "",
        );
        assert!(report.source.contains("plain-page(\"Alpha\")"));
        assert!(report.source.contains("plain-page(\"Beta\")"));
        let breaks = report.source.matches("#pagebreak()").count();
        assert!(breaks >= 1, "expected pagebreak between pages, got {breaks}");
    }
}
