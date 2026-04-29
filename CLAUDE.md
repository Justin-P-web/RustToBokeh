# CLAUDE.md — RustToBokeh

Interactive Bokeh dashboards from Rust + Polars. Two render paths produce equivalent HTML.

## Architecture

* **Native (default, `bokeh-inline`):** Polars → `bokeh_native::render_native_dashboard` → Bokeh JSON → self-contained HTML. No runtime Python.
* **Python (`python` feature):** Polars → Arrow IPC → PyO3 → embedded `python/render.py` → Bokeh → HTML.

Each `Page` becomes one HTML file with shared nav. Output is offline (Bokeh JS/CSS inlined or `BokehResources::Cdn`).

## Project Layout

| Path | Role |
| :--- | :--- |
| `src/lib.rs` | Crate root, `NavStyle`, `serialize_df`, re-exports. |
| `src/dashboard.rs` | `Dashboard` builder: register frames, add pages, dispatch render. |
| `src/pages.rs` | `Page` / `PageBuilder`. |
| `src/handle.rs` | `DfHandle` typed reference to a registered frame. |
| `src/modules.rs` | Page modules: `Paragraph`, `Table`, `StatGrid`. |
| `src/charts/` | `ChartConfig`/`ChartSpec`/builders + `customization/` (palette, axis, tooltip, filters, time scale). |
| `src/stats.rs` | Histogram / box-plot / outlier math. |
| `src/error.rs` | `ChartError`. |
| `src/validator.rs` | Pre-render checks. |
| `src/prelude.rs` | Common re-exports. |
| `src/bokeh_native/` | Pure-Rust Bokeh JSON + HTML renderer (`model`, `document`, `figure`, `charts`, `filters`, `axis`, `palette`, `source`, `nav`, `html`, `page`). |
| `src/render/` | PyO3 bridge (`python` feature) — `chart_config` + `module` serializers. |
| `src/python_config.rs` | Vendored interpreter discovery. |
| `python/render.py`, `templates/chart.html` | Embedded via `include_str!`. |
| `src/bin/example_dashboard/`, `src/bin/sensor_report/` | Demo binaries. |
| `tests/dashboard_output.rs` | End-to-end integration test. |
| `scripts/setup_vendor.{sh,ps1}` | Populate `vendor/` with Python + Bokeh assets. |

## Build & Development

* Vendor: `bash scripts/setup_vendor.sh` (Windows: `setup_vendor.ps1`).
* `.cargo/config.toml` points `PYO3_PYTHON` at the vendored interpreter; `build.rs` copies DLLs on Windows.
* Recompile after editing `python/render.py` or `templates/chart.html` (`include_str!`).

Commands:
* Demo: `cargo run --bin example-dashboard --release`
* Sensor demo: `cargo run --bin sensor-report --release`
* Test: `cargo nextest run` (never `cargo test`)
* Python path: append `--features python`.

## Quality Standards

**Docs:** `//!` on every module (note Rust↔Python and Rust↔Bokeh-JSON boundaries). `///` on every public item — units, coordinate systems, mapped Bokeh property. `# Examples` on every builder. Prefer doc-tests so examples can't rot.

**Tests:** `mod tests` in logic-heavy files (`stats`, `lib`, `dashboard`, `validator`). Table-driven for `ChartConfig` → output. Round-trip `serialize_df` through Arrow IPC. `tests/dashboard_output.rs` exercises both render paths.

## Rules

**Rust:**
* Collect lazy Polars frames before serializing.
* All PyO3 calls inside `Python::with_gil`.
* Same `source_key` on a page → shared CDS (linked selection). Multiple filters on one source → automatic `IntersectionFilter`.
* New filter: `FilterConfig` variant + factory in `customization/filters.rs` + native handler in `bokeh_native/filters/` + Python handler in `render.py`.
* Prefer enums over strings, builders over ad-hoc structs.
* `bokeh_native` is a **public API** consumed by downstream report tools — every `pub` item there must carry rustdoc, including helper constructors, `build_*` factories, and re-exported model primitives.

**Python & vendor:** Add deps to `requirements.txt`, re-run `setup_vendor.sh`. Never edit `vendor/`. Python path stays `INLINE`. Filter serialization extends `build_filter_objects()` in `render.py`.

**Aesthetic:** Industrial / lab-instrument — flat cards, dense rhythmic spacing, no gradients/glow. Colors: `light-dark()` + OKLCH only, one sharp accent, no hex. Humanist sans body, tabular numerics for data.

## Workflow

Branch `claude/<description>`. PR to `main` only. Never hand-edit `Cargo.lock` or `vendor/`. Output must remain a single self-contained offline HTML per page.
