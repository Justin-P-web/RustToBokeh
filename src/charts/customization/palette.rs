/// A color palette used to assign colors to groups or series.
///
/// Used by grouped-bar and line charts to override the default seaborn-style
/// color cycle.
///
/// # Examples
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// // Bokeh built-in named palette
/// let p = PaletteSpec::Named("Plasma256".into());
///
/// // Custom hex colors — cycled when fewer colors than groups
/// let p = PaletteSpec::Custom(vec!["#e74c3c".into(), "#3498db".into()]);
/// ```
pub enum PaletteSpec {
    /// One of Bokeh's built-in named palettes (e.g. `"Category10"`,
    /// `"Category20"`, `"Viridis256"`, `"Plasma256"`).
    Named(String),
    /// A list of hex color strings (e.g. `"#4C72B0"`).  Cycled when fewer
    /// entries are supplied than there are groups or series.
    Custom(Vec<String>),
}
