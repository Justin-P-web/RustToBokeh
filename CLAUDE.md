# CLAUDE.md — RustToBokeh

**Core Pipeline:** Rust (Polars) -> Arrow IPC -> PyO3 (Embedded Python) -> Bokeh -> Self-contained HTML.

---

## Architecture & Flow

1. **Data Prep (Rust):** Build Polars DataFrames, compute stats (histogram/box-plot) via `src/stats.rs`, and register via `Dashboard::add_df()`.
2. **Definition (Rust):** Define `Page`s with `ChartSpec`/`FilterSpec`. Use `ChartSpecBuilder` for layout (`.at()`, `.filtered()`, `.dimensions()`).
3. **Serialization:** `render() -> serialize_df` converts DataFrames to Arrow IPC.
4. **Python Bridge:** `src/render.rs` takes the GIL, executes `python/render.py` (via `include_str!`).
5. **Output:** Python deserializes, builds Bokeh models with `CDSView` + `IntersectionFilter`, and injects JS/CSS via `bokeh.resources.INLINE` into `templates/chart.html`. Result is zero-dependency offline HTML.

---

## Project Layout

| Path | Responsibility |
| :--- | :--- |
| `src/lib.rs` | Core `Dashboard` logic, IPC serialization, and `NavStyle`. |
| `src/stats.rs` | Math for histograms, box-plots, and outliers. |
| `src/render.rs` | PyO3 bridge and Python script execution. |
| `src/python_config.rs` | Logic for discovery of the vendored Python runtime. |
| `src/charts/charts/` | `ChartConfig` variants (GroupedBar, Line, Scatter, Pie, etc.). |
| `src/charts/customization/` | Palettes, TimeScale, Tooltips, and `FilterConfig`. |
| `python/render.py` | Embedded Python logic for Bokeh generation. |
| `scripts/setup_vendor.sh` | Shell script to populate `vendor/` with Python + dependencies. |

---

## Build & Development

### Build Requirements
* **Vendoring:** `bash scripts/setup_vendor.sh` must be run to populate `vendor/`.
* **Config:** `.cargo/config.toml` must point `PYO3_PYTHON` to the vendored interpreter.
* **Windows:** `build.rs` is used to copy necessary DLLs for the runtime.

### Key Commands
* **Run Demo:** `cargo run --bin example-dashboard --release`
* **Test:** `cargo nextest run`
* **Recompilation:** Recompile the Rust binary after any change to `python/render.py` or `templates/chart.html` due to `include_str!`.

---

## Quality & Reliability Standards

### Documentation (Rustdoc)
* **Module Level:** Use `//!` to explain how a module interacts with the Python/Bokeh layer.
* **Public API:** Every public struct and method requires `///` comments. 
* **Mandatory Examples:** Use `# Examples` blocks in `ChartSpecBuilder` methods to demonstrate valid layout/filter combinations.
* **Invariant Docs:** Explicitly document units (pixels/normalized) and coordinate systems for chart layout.
* **Python Boundary:** Document where a Rust field maps to a specific Bokeh model property.

### Testing Strategy
* **Unit Tests:** Logic-heavy files (`src/stats.rs`, `src/lib.rs`) must contain a `mod tests` block.
* **Serialization Checks:** Test that `serialize_df` produces valid Arrow IPC schema recognized by Python-Polars.
* **Table-Driven Tests:** Use for `ChartConfig` serialization to ensure specific Rust enums map to the correct JSON/Python inputs.
* **Integration Tests:** Use `tests/integration_render.rs` to verify the full PyO3 pipeline generates a non-empty HTML string.
* **Doc-Tests:** Prioritize doc-tests for builder patterns to ensure examples remain valid as the API evolves.

---

## Rules & Patterns

### Rust Design
* **Efficiency:** Always `.collect()` lazy Polars DataFrames before serialization.
* **Safety:** All PyO3 interactions must occur within `Python::with_gil`.
* **Linking:** Shared `source_key` on a single page creates shared CDS (linked selection). Multiple filters on one source automatically apply `IntersectionFilter`.
* **Expansion:** New filters require a `FilterConfig` variant, a factory in `filters.rs`, and a handler in `render.py`.

### Python & Vendoring
* **Dependencies:** Add to `requirements.txt` and re-run `setup_vendor.sh`. Do not edit `vendor/` manually.
* **Resources:** Keep Bokeh resource mode as `INLINE`. No CDN fallbacks allowed.
* **State:** Use `build_filter_objects()` in `render.py` for new filter serialization branches.

### Utilitarian Design Aesthetic
* **Visuals:** Industrial/Lab-instrument aesthetic. Flat cards, dense rhythmic spacing. No gradients/glow.
* **Colors:** Use `light-dark()` with OKLCH values. No hex codes. One sharp accent color only.
* **Typography:** Humanist sans for body; Tabular numeric faces for all data/tables.

---

## Git & Workflow
* **Branching:** Use `claude/<description>`. PR to `main` only.
* **Files:** Never manually edit `Cargo.lock` or the contents of the `vendor/` directory.

> **Note:** The final output must remain a single, self-contained HTML file that functions entirely offline.