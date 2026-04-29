//! Core Bokeh JSON model types.
//!
//! Bokeh 3.x serializes its document model to a specific JSON schema.
//! This module provides `BokehValue` and `BokehObject` which serialize
//! to that schema.
//!
//! Key serialization rules:
//! - `Map` → `{"type":"map","entries":[[k,v],...]}`  (NOT a plain JSON object)
//! - `Ref(id)` → `{"id":"p1001"}` (cross-reference to another object)
//! - `Field(col)` → `{"type":"field","field":"col"}`
//! - `FieldTransform` → `{"type":"field","field":"col","transform":{...}}`
//! - `Value(v)` → `{"type":"value","value":v}`
//! - `NaN` → `{"type":"number","value":"nan"}`
//! - `Object` → `{"type":"object","name":"TypeName","id":"p1","attributes":{...}}`

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

// ── BokehValue ───────────────────────────────────────────────────────────────

/// A value that can appear in a Bokeh model's attributes.
#[derive(Clone)]
pub enum BokehValue {
    /// JSON `null`.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit float. Use [`BokehValue::number`] when the source value may be
    /// `NaN`.
    Float(f64),
    /// Special NaN → `{"type":"number","value":"nan"}`
    NaN,
    /// JSON string.
    Str(String),
    /// JSON array of values.
    Array(Vec<BokehValue>),
    /// `{"type":"map","entries":[[k,v],...]}`
    Map(Vec<(String, BokehValue)>),
    /// `{"type":"field","field":"col"}`
    Field(String),
    /// `{"type":"field","field":"col","transform":{...}}`
    FieldTransform { field: String, transform: Box<BokehValue> },
    /// `{"type":"value","value":v}`
    Value(Box<BokehValue>),
    /// Full inline object with type/name/id/attributes.
    Object(Box<BokehObject>),
    /// Cross-reference: `{"id":"p1001"}`
    Ref(String),
}

impl BokehValue {
    /// Construct a string value (`BokehValue::Str`).
    pub fn str(s: impl Into<String>) -> Self {
        BokehValue::Str(s.into())
    }

    /// Construct a column-field reference (`{"type":"field","field":"col"}`).
    /// Use this when a glyph property reads from a `ColumnDataSource` column
    /// rather than a fixed value.
    pub fn field(col: impl Into<String>) -> Self {
        BokehValue::Field(col.into())
    }

    /// Wrap a value as `{"type":"value","value":v}` — the explicit form Bokeh
    /// expects when distinguishing fixed values from field references.
    pub fn value_of(v: BokehValue) -> Self {
        BokehValue::Value(Box::new(v))
    }

    /// Wrap a [`BokehObject`] inline as a value (`BokehValue::Object`).
    pub fn obj(o: BokehObject) -> Self {
        BokehValue::Object(Box::new(o))
    }

    /// Construct a cross-reference (`{"id":"p1001"}`) to another model that
    /// is defined elsewhere in the document.
    pub fn ref_of(id: impl Into<String>) -> Self {
        BokehValue::Ref(id.into())
    }

    /// Construct a field reference with a transform applied
    /// (`{"type":"field","field":"col","transform":{...}}`). `transform` must
    /// serialize to a Bokeh transform model such as `LinearColorMapper` or
    /// `CustomJSTransform`.
    pub fn field_transform(field: impl Into<String>, transform: BokehValue) -> Self {
        BokehValue::FieldTransform {
            field: field.into(),
            transform: Box::new(transform),
        }
    }

    /// Wrap a float as a Bokeh numeric value spec — handles NaN specially.
    pub fn number(v: f64) -> Self {
        if v.is_nan() { BokehValue::NaN } else { BokehValue::Float(v) }
    }
}

impl Serialize for BokehValue {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self {
            BokehValue::Null => ser.serialize_none(),
            BokehValue::Bool(b) => ser.serialize_bool(*b),
            BokehValue::Int(n) => ser.serialize_i64(*n),
            BokehValue::Float(f) => ser.serialize_f64(*f),
            BokehValue::NaN => {
                let mut m = ser.serialize_map(Some(2))?;
                m.serialize_entry("type", "number")?;
                m.serialize_entry("value", "nan")?;
                m.end()
            }
            BokehValue::Str(s) => ser.serialize_str(s),
            BokehValue::Array(arr) => {
                let mut seq = ser.serialize_seq(Some(arr.len()))?;
                for v in arr {
                    seq.serialize_element(v)?;
                }
                seq.end()
            }
            BokehValue::Map(entries) => {
                let mut m = ser.serialize_map(Some(2))?;
                m.serialize_entry("type", "map")?;
                m.serialize_entry("entries", &MapEntries(entries))?;
                m.end()
            }
            BokehValue::Field(col) => {
                let mut m = ser.serialize_map(Some(2))?;
                m.serialize_entry("type", "field")?;
                m.serialize_entry("field", col)?;
                m.end()
            }
            BokehValue::FieldTransform { field, transform } => {
                let mut m = ser.serialize_map(Some(3))?;
                m.serialize_entry("type", "field")?;
                m.serialize_entry("field", field)?;
                m.serialize_entry("transform", transform.as_ref())?;
                m.end()
            }
            BokehValue::Value(v) => {
                let mut m = ser.serialize_map(Some(2))?;
                m.serialize_entry("type", "value")?;
                m.serialize_entry("value", v.as_ref())?;
                m.end()
            }
            BokehValue::Object(o) => o.serialize(ser),
            BokehValue::Ref(id) => {
                let mut m = ser.serialize_map(Some(1))?;
                m.serialize_entry("id", id)?;
                m.end()
            }
        }
    }
}

/// Helper to serialize `Vec<(String, BokehValue)>` as a JSON array of 2-element arrays.
struct MapEntries<'a>(&'a Vec<(String, BokehValue)>);

impl Serialize for MapEntries<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut seq = ser.serialize_seq(Some(self.0.len()))?;
        for (k, v) in self.0 {
            seq.serialize_element(&(k, v))?;
        }
        seq.end()
    }
}

// ── BokehObject ──────────────────────────────────────────────────────────────

/// A Bokeh model object with a type name, unique ID, and attributes.
#[derive(Clone)]
pub struct BokehObject {
    /// Bokeh model type name, e.g. `"Figure"`, `"VBar"`, `"BooleanFilter"`.
    pub name: &'static str,
    /// Unique model ID, e.g. `"p1001"`.
    pub id: String,
    /// Model attributes as name→value pairs.
    pub attributes: Vec<(String, BokehValue)>,
}

impl BokehObject {
    /// Create a Bokeh model object with a type `name` and unique `id` and no
    /// attributes. Chain [`attr`](Self::attr) to add attributes.
    pub fn new(name: &'static str, id: String) -> Self {
        BokehObject { name, id, attributes: Vec::new() }
    }

    /// Create a Bokeh model object with attributes provided up front. Equivalent
    /// to [`new`](Self::new) followed by repeated [`attr`](Self::attr) calls,
    /// but cheaper when many attributes are already known.
    pub fn with_attrs(
        name: &'static str,
        id: String,
        attrs: Vec<(&'static str, BokehValue)>,
    ) -> Self {
        BokehObject {
            name,
            id,
            attributes: attrs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, key: &str, val: BokehValue) -> Self {
        self.attributes.push((key.to_string(), val));
        self
    }

    /// Get a cross-reference to this object (just its ID).
    pub fn to_ref(&self) -> BokehValue {
        BokehValue::Ref(self.id.clone())
    }

    /// Get this object as a `BokehValue::Object`.
    pub fn into_value(self) -> BokehValue {
        BokehValue::Object(Box::new(self))
    }
}

impl Serialize for BokehObject {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let attr_count = if self.attributes.is_empty() { 3 } else { 4 };
        let mut m = ser.serialize_map(Some(attr_count))?;
        m.serialize_entry("type", "object")?;
        m.serialize_entry("name", self.name)?;
        m.serialize_entry("id", &self.id)?;
        if !self.attributes.is_empty() {
            m.serialize_entry("attributes", &ObjAttributes(&self.attributes))?;
        }
        m.end()
    }
}

/// Helper to serialize `Vec<(String, BokehValue)>` as a JSON object.
struct ObjAttributes<'a>(&'a Vec<(String, BokehValue)>);

impl Serialize for ObjAttributes<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut m = ser.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0 {
            m.serialize_entry(k, v)?;
        }
        m.end()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn to_json(v: &BokehValue) -> String {
        serde_json::to_string(v).unwrap()
    }

    #[test]
    fn map_uses_entries_format() {
        let v = BokehValue::Map(vec![
            ("x".into(), BokehValue::Array(vec![BokehValue::Int(1), BokehValue::Int(2)])),
        ]);
        let j = to_json(&v);
        assert!(j.contains("\"type\":\"map\""));
        assert!(j.contains("\"entries\""));
        assert!(!j.contains("{\"x\":"));
    }

    #[test]
    fn ref_serializes_to_id_only() {
        let v = BokehValue::Ref("p1234".into());
        assert_eq!(to_json(&v), r#"{"id":"p1234"}"#);
    }

    #[test]
    fn field_serializes_correctly() {
        let v = BokehValue::Field("revenue".into());
        assert_eq!(to_json(&v), r#"{"type":"field","field":"revenue"}"#);
    }

    #[test]
    fn value_spec_serializes_correctly() {
        let v = BokehValue::value_of(BokehValue::Float(0.9));
        let j = to_json(&v);
        assert!(j.contains("\"type\":\"value\""));
        assert!(j.contains("\"value\":0.9"));
    }

    #[test]
    fn nan_serializes_as_string() {
        let v = BokehValue::NaN;
        assert_eq!(to_json(&v), r#"{"type":"number","value":"nan"}"#);
    }

    #[test]
    fn object_serializes_with_type_name_id() {
        let o = BokehObject::new("PanTool", "p42".into());
        let j = serde_json::to_string(&o).unwrap();
        assert!(j.contains("\"type\":\"object\""));
        assert!(j.contains("\"name\":\"PanTool\""));
        assert!(j.contains("\"id\":\"p42\""));
    }

    #[test]
    fn object_with_attrs_serializes_attributes() {
        let o = BokehObject::new("Slider", "p99".into())
            .attr("title", BokehValue::Str("Test".into()));
        let j = serde_json::to_string(&o).unwrap();
        assert!(j.contains("\"attributes\""));
        assert!(j.contains("\"title\""));
    }

    #[test]
    fn field_transform_serializes_with_three_keys() {
        let transform = BokehObject::new("CategoricalColorMapper", "p5".into())
            .attr("palette", BokehValue::Array(vec![BokehValue::Str("#fff".into())]))
            .attr("factors", BokehValue::Array(vec![BokehValue::Str("A".into())]));
        let v = BokehValue::field_transform("col", transform.into_value());
        let j = to_json(&v);
        assert!(j.contains("\"type\":\"field\""));
        assert!(j.contains("\"field\":\"col\""));
        assert!(j.contains("\"transform\""));
    }
}
