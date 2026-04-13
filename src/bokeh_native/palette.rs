//! Color palette resolution for native Bokeh rendering.

use crate::charts::PaletteSpec;

/// Default 10-color palette matching the Python renderer.
pub const DEFAULT_PALETTE: &[&str] = &[
    "#4C72B0", "#DD8452", "#55A868", "#C44E52",
    "#8172B3", "#937860", "#DA8BC3", "#8C8C8C",
    "#CCB974", "#64B5CD",
];

// Hard-coded named palettes from Bokeh 3.9.0
const CATEGORY10: &[&str] = &[
    "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd",
    "#8c564b", "#e377c2", "#7f7f7f", "#bcbd22", "#17becf",
];

const CATEGORY20: &[&str] = &[
    "#1f77b4", "#aec7e8", "#ff7f0e", "#ffbb78", "#2ca02c",
    "#98df8a", "#d62728", "#ff9896", "#9467bd", "#c5b0d5",
    "#8c564b", "#c49c94", "#e377c2", "#f7b6d2", "#7f7f7f",
    "#c7c7c7", "#bcbd22", "#dbdb8d", "#17becf", "#9edae5",
];

const VIRIDIS8: &[&str] = &[
    "#440154", "#3b528b", "#21918c", "#5ec962",
    "#fde725", "#440154", "#3b528b", "#21918c",
];

const PLASMA8: &[&str] = &[
    "#0d0887", "#6a00a8", "#b12a90", "#e16462",
    "#fca636", "#f0f921", "#0d0887", "#6a00a8",
];

/// Resolve a `PaletteSpec` to exactly `n` hex color strings.
///
/// If more colors are needed than the palette provides, colors are cycled.
pub fn resolve_palette(spec: Option<&PaletteSpec>, n: usize) -> Vec<String> {
    let base: &[&str] = match spec {
        None => DEFAULT_PALETTE,
        Some(PaletteSpec::Named(name)) => named_palette(name),
        Some(PaletteSpec::Custom(colors)) => {
            return cycle_colors_owned(colors, n);
        }
    };
    cycle_colors(base, n)
}

fn named_palette(name: &str) -> &'static [&'static str] {
    match name {
        "Category10" | "category10" => CATEGORY10,
        "Category20" | "category20" => CATEGORY20,
        "Viridis" | "Viridis8" => VIRIDIS8,
        "Plasma" | "Plasma8" => PLASMA8,
        _ => DEFAULT_PALETTE,
    }
}

fn cycle_colors(base: &[&str], n: usize) -> Vec<String> {
    if n == 0 { return Vec::new(); }
    (0..n).map(|i| base[i % base.len()].to_string()).collect()
}

fn cycle_colors_owned(base: &[String], n: usize) -> Vec<String> {
    if n == 0 { return Vec::new(); }
    (0..n).map(|i| base[i % base.len()].clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_cycles_when_n_exceeds_length() {
        let colors = resolve_palette(None, 12);
        assert_eq!(colors.len(), 12);
        assert_eq!(colors[0], colors[10]);
    }

    #[test]
    fn named_category10_returns_correct_first_color() {
        let colors = resolve_palette(Some(&PaletteSpec::Named("Category10".into())), 3);
        assert_eq!(colors[0], "#1f77b4");
    }

    #[test]
    fn custom_palette_cycles() {
        let spec = PaletteSpec::Custom(vec!["#aaa".into(), "#bbb".into()]);
        let colors = resolve_palette(Some(&spec), 5);
        assert_eq!(colors, vec!["#aaa", "#bbb", "#aaa", "#bbb", "#aaa"]);
    }
}
