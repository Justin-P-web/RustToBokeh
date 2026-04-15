//! X/Y range builders (Range1d, DataRange1d, FactorRange) with AxisConfig support.

use crate::charts::AxisConfig;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::{XRangeKind, YRangeKind};

pub(super) fn build_x_range(
    id_gen: &mut IdGen,
    kind: XRangeKind,
    cfg: Option<&AxisConfig>,
) -> (BokehValue, String) {
    match kind {
        XRangeKind::Factor(factors) => {
            let id = id_gen.next();
            let obj = BokehObject::new("FactorRange", id.clone())
                .attr("factors", BokehValue::Array(factors));
            (obj.into_value(), id)
        }
        XRangeKind::Numeric { start, end } => {
            let id = id_gen.next();
            let mut obj = BokehObject::new("DataRange1d", id.clone());
            if start != 0.0 || end != 0.0 {
                obj = BokehObject::new("Range1d", id.clone())
                    .attr("start", BokehValue::Float(start))
                    .attr("end", BokehValue::Float(end));
            }
            if let Some(cfg) = cfg {
                obj = apply_range_config(obj, cfg);
            }
            (obj.into_value(), id)
        }
        XRangeKind::Datetime { start, end } => {
            let id = id_gen.next();
            let obj = BokehObject::new("Range1d", id.clone())
                .attr("start", BokehValue::Float(start))
                .attr("end", BokehValue::Float(end));
            (obj.into_value(), id)
        }
        XRangeKind::ExistingId(existing_id) => {
            // Emit a cross-reference ({"id": "..."}) — NOT a full inline object.
            // The Range1d is already a document root; embedding it inline here
            // would overwrite its attributes (start, end, callbacks) with an
            // empty object, breaking the shared x-axis synchronisation.
            (BokehValue::Ref(existing_id.clone()), existing_id)
        }
        XRangeKind::DataRange => {
            let id = id_gen.next();
            let obj = BokehObject::new("DataRange1d", id.clone());
            (obj.into_value(), id)
        }
    }
}

pub(super) fn build_y_range(
    id_gen: &mut IdGen,
    kind: YRangeKind,
    cfg: Option<&AxisConfig>,
) -> (BokehValue, String) {
    match kind {
        YRangeKind::DataRange => {
            let id = id_gen.next();
            let mut obj = BokehObject::new("DataRange1d", id.clone());
            if let Some(cfg) = cfg {
                obj = apply_range_config(obj, cfg);
            }
            (obj.into_value(), id)
        }
        YRangeKind::Numeric { start, end } => {
            let id = id_gen.next();
            let obj = BokehObject::new("Range1d", id.clone())
                .attr("start", BokehValue::Float(start))
                .attr("end", BokehValue::Float(end));
            (obj.into_value(), id)
        }
        YRangeKind::Factor(factors) => {
            let id = id_gen.next();
            let obj = BokehObject::new("FactorRange", id.clone())
                .attr("factors", BokehValue::Array(factors));
            (obj.into_value(), id)
        }
    }
}

fn apply_range_config(mut obj: BokehObject, cfg: &AxisConfig) -> BokehObject {
    if let Some(start) = cfg.start {
        obj = obj.attr("start", BokehValue::Float(start));
    }
    if let Some(end) = cfg.end {
        obj = obj.attr("end", BokehValue::Float(end));
    }
    if let (Some(bmin), Some(bmax)) = (cfg.bounds_min, cfg.bounds_max) {
        obj = obj.attr(
            "bounds",
            BokehValue::Array(vec![BokehValue::Float(bmin), BokehValue::Float(bmax)]),
        );
    }
    obj
}
