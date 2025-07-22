
use bevy_math::prelude::{Vec3, Quat, EaseFunction};
use bevy_transform::prelude::Transform;
use bevy_timeline_runtime::{AnimatableValue, TimelineError};

#[derive(Clone)]
pub struct Scale {
    ease: EaseFunction,
    val: Vec3
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            ease: EaseFunction::Linear,
            val: Default::default()
        }
    }
}

impl AnimatableValue for Scale {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.scale = start.0 + (end.0 - start.0) * s;
        } else {
            out.scale = start.0;
        }
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        let values = value.as_array().ok_or( TimelineError::IncorrectValueType("`Scale` keyframe must be [f32;3] array") )?;
        if values.len() != 3 {
            return Err( TimelineError::IncorrectValueType("`Scale` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( TimelineError::IncorrectValueType("`Scale`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( TimelineError::IncorrectValueType("`Scale`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( TimelineError::IncorrectValueType("`Scale`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let scale = self.0;
        Value::Array(vec![Value::from(scale.x), Value::from(scale.y), Value::from(scale.z)])
    }

    fn typ() -> &'static str {
        "Scale"
    }
}

#[derive(Default,Clone)]
pub struct Rotation(Quat);

impl AnimatableValue for bevy_timeline_runtime::Rotation {
    type Target = Transform;

    fn interpolate(s: f32, prev:Self, next: Option<Self>, out: &mut Self::Target) {
        if let Some(next) = next {
            out.rotation = prev.0.slerp( next.0, s );
        } else {
            out.rotation = prev.0;
        }
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        let values = value.as_array().ok_or( TimelineError::IncorrectValueType("`Rotation` keyframe must be [f32;3] array") )?;
        if values.len() != 4 {
            return Err( TimelineError::IncorrectValueType("`Rotation` keyframe must be [f32;4] array") );
        }

        Ok( Self(Quat::from_array([
            values[0].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[3].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[3] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let rotation = self.0;
        Value::Array(vec![Value::from(rotation.x), Value::from(rotation.y), Value::from(rotation.z), Value::from(rotation.w) ])
    }

    fn typ() -> &'static str {
        "Rotation"
    }
}


#[derive(Default,Clone)]
pub struct Translation(Vec3);

impl AnimatableValue for bevy_timeline_runtime::Translation {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.translation = start.0 + (end.0 - start.0) * s;
            // println!("translation(0) {}", out.translation);
        } else {
            out.translation = start.0;
            // println!("translation(1) {}", out.translation);
        }
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        let values = value.as_array().ok_or( TimelineError::IncorrectValueType("`Translation` keyframe must be [f32;3] array") )?;
        if values.len() != 3 {
            return Err( TimelineError::IncorrectValueType("`Translation` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( TimelineError::IncorrectValueType("`Translation`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( TimelineError::IncorrectValueType("`Translation`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( TimelineError::IncorrectValueType("`Translation`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let translation = self.0;
        Value::Array(vec![Value::from(translation.x), Value::from(translation.y), Value::from(translation.z)])
    }

    fn typ() -> &'static str {
        "Translation"
    }
}
