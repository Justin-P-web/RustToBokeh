//! ColumnDataSource builder from Polars DataFrames.

use polars::prelude::*;

use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

/// Build a Bokeh `ColumnDataSource` from a Polars `DataFrame`.
///
/// The `data` attribute is serialized as a Bokeh map type:
/// `{"type":"map","entries":[["col1",[v1,v2,...]],...]}`.
///
/// Extra columns (e.g. `_fill_color`) can be injected before calling this
/// function by adding them to the DataFrame.
pub fn build_column_data_source(id_gen: &mut IdGen, df: &DataFrame) -> BokehObject {
    let cds_id = id_gen.next();
    let sel_id = id_gen.next();
    let policy_id = id_gen.next();

    let mut entries: Vec<(String, BokehValue)> = Vec::new();
    for col in df.columns() {
        let col_name = col.name().to_string();
        let values = series_to_bokeh_array(col);
        entries.push((col_name, values));
    }

    let selection = BokehObject::with_attrs(
        "Selection",
        sel_id,
        vec![
            ("indices", BokehValue::Array(vec![])),
            ("line_indices", BokehValue::Array(vec![])),
        ],
    );

    let policy = BokehObject::new("UnionRenderers", policy_id);

    BokehObject::new("ColumnDataSource", cds_id)
        .attr("selected", selection.into_value())
        .attr("selection_policy", policy.into_value())
        .attr("data", BokehValue::Map(entries))
}

/// Convert a Polars Column to a `BokehValue::Array`.
fn series_to_bokeh_array(series: &Column) -> BokehValue {
    let values: Vec<BokehValue> = match series.dtype() {
        DataType::Float32 => series
            .f32()
            .unwrap()
            .into_iter()
            .map(|v| v.map_or(BokehValue::Null, |x| BokehValue::Float(x as f64)))
            .collect(),
        DataType::Float64 => series
            .f64()
            .unwrap()
            .into_iter()
            .map(|v| v.map_or(BokehValue::Null, BokehValue::Float))
            .collect(),
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
            let cast = series.cast(&DataType::Int64).unwrap_or_else(|_| series.clone());
            cast.i64()
                .unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, BokehValue::Int))
                .collect()
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            let cast = series.cast(&DataType::Int64).unwrap_or_else(|_| series.clone());
            cast.i64()
                .unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, BokehValue::Int))
                .collect()
        }
        DataType::Boolean => series
            .bool()
            .unwrap()
            .into_iter()
            .map(|v| v.map_or(BokehValue::Null, BokehValue::Bool))
            .collect(),
        DataType::String => series
            .str()
            .unwrap()
            .into_iter()
            .map(|v| v.map_or(BokehValue::Null, |s| BokehValue::Str(s.to_string())))
            .collect(),
        DataType::Categorical(_, _) | DataType::Enum(_, _) => {
            let cast = series.cast(&DataType::String).unwrap_or_else(|_| series.clone());
            cast.str()
                .unwrap()
                .into_iter()
                .map(|v| v.map_or(BokehValue::Null, |s| BokehValue::Str(s.to_string())))
                .collect()
        }
        _ => {
            // Fallback: try casting to f64
            let cast = series.cast(&DataType::Float64).unwrap_or_else(|_| series.clone());
            if let Ok(ca) = cast.f64() {
                ca.into_iter()
                    .map(|v| v.map_or(BokehValue::Null, BokehValue::Float))
                    .collect()
            } else {
                vec![]
            }
        }
    };
    BokehValue::Array(values)
}

/// Extract a column from a DataFrame as a Vec<f64>, returning an error if missing.
pub fn get_f64_column(df: &DataFrame, col: &str) -> Result<Vec<f64>, String> {
    let series = df
        .column(col)
        .map_err(|_| format!("column '{col}' not found in DataFrame"))?;
    let cast = series
        .cast(&DataType::Float64)
        .map_err(|e| format!("cannot cast column '{col}' to f64: {e}"))?;
    Ok(cast
        .f64()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect())
}

/// Extract a column from a DataFrame as a Vec<String>.
pub fn get_str_column(df: &DataFrame, col: &str) -> Result<Vec<String>, String> {
    let series = df
        .column(col)
        .map_err(|_| format!("column '{col}' not found in DataFrame"))?;
    let cast = series
        .cast(&DataType::String)
        .map_err(|e| format!("cannot cast column '{col}' to String: {e}"))?;
    Ok(cast
        .str()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap_or("").to_string())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn cds_has_data_map() {
        let df = df![
            "x" => [1.0f64, 2.0, 3.0],
            "y" => [4.0f64, 5.0, 6.0],
        ]
        .unwrap();
        let mut id_gen = IdGen::new();
        let cds = build_column_data_source(&mut id_gen, &df);
        assert_eq!(cds.name, "ColumnDataSource");
        // Find data attribute
        let data = cds.attributes.iter().find(|(k, _)| k == "data");
        assert!(data.is_some());
        if let Some((_, BokehValue::Map(entries))) = data {
            let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"x"));
            assert!(keys.contains(&"y"));
        } else {
            panic!("data should be a BokehValue::Map");
        }
    }

    #[test]
    fn string_column_serializes_as_str_array() {
        let df = df!["name" => ["Alice", "Bob"]].unwrap();
        let mut id_gen = IdGen::new();
        let cds = build_column_data_source(&mut id_gen, &df);
        let data = cds.attributes.iter().find(|(k, _)| k == "data").unwrap();
        if let (_, BokehValue::Map(entries)) = data {
            let name_entry = entries.iter().find(|(k, _)| k == "name").unwrap();
            if let (_, BokehValue::Array(vals)) = name_entry {
                assert!(matches!(&vals[0], BokehValue::Str(s) if s == "Alice"));
            }
        }
    }
}
