use std::ops::Deref;
use bevy_math::Curve;
use bevy_math::prelude::{Vec3, Quat, EaseFunction};
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde_json::{json, Value};

use bevy_transform::prelude::Transform;

pub type TransformSet = (Scale, Rotation, Translation);

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct EaseFunctionWrap(EaseFunction);

impl Deref for EaseFunctionWrap {
    type Target = EaseFunction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for EaseFunctionWrap {
    fn default() -> Self {
        Self( EaseFunction::Linear )
    }
}

#[derive(Default,Clone)]
pub struct Scale {
    ease: EaseFunctionWrap,
    v3: Vec3
}


impl AnimatableValue for Scale {
    type Target = Transform;

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
}

#[derive(Default,Clone)]
pub struct Rotation {
    ease: EaseFunctionWrap,
    quat: Quat
}

impl AnimatableValue for Rotation {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(next) = end {
            let s = start.ease.sample_unchecked(s);
            out.rotation = start.quat.slerp( next.quat, s );
        } else {
            out.rotation = start.quat;
        }
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self {
            ease: serde_json::from_value( value["ease"].clone() )?,
            quat: Quat::from_array( value.get_fixed_array_f32("quat")? )
        } )
    }

    fn to_value(&self) -> Value {
        json!({
            "ease" : self.ease,
            "v3" : Value::from( vec![self.quat.x, self.quat.y, self.quat.z, self.quat.w] )
        })
    }
}


#[derive(Default,Clone)]
pub struct Translation {
    ease: EaseFunctionWrap,
    v3: Vec3
}


impl AnimatableValue for Translation {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.translation = start.v3 + (end.v3 - start.v3) * start.ease.sample_unchecked(s);
        } else {
            out.translation = start.v3;
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
}