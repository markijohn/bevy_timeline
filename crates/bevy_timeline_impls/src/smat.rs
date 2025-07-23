use serde_json::{json, Value};
use bevy_timeline_runtime::{AnimatableValue, TimelineError};
use crate::transform::EaseFunctionWrap;

use bevy_pbr::StandardMaterial;

#[derive(Default,Clone)]
pub struct Scale {
    ease: EaseFunctionWrap,
    v3: Vec3
}


impl AnimatableValue for Scale {
    type Target = StandardMaterial;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.scale = start.v3 + (end.v3 - start.v3) * start.ease.sample_unchecked(s);
        } else {
            out.scale = start.v3;
        }
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self {
            ease: serde_json::from_value( value["ease"].clone() )?,
            v3: Vec3::from_array(value.get_fixed_array_f32("v3")? )
        })
    }

    fn to_value(&self) -> Value {
        json!({
            "ease" : self.ease,
            "v3" : Value::from( vec![self.v3.x, self.v3.y, self.v3.z] )
        })
    }

    fn typ() -> &'static str {
        "Scale"
    }
}