use std::borrow::Cow;
use bevy_ecs::component::Mutable;
use bevy_reflect::TypePath;
use bevy_ecs::prelude::Component;
use bevy_ecs::query::QueryData;
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

use serde_json::{json,Value};

pub trait AnimatableValue:TypePath+Sized {
    type Target: Component<Mutability=Mutable>;
    fn interpolate(&self, s:f32, next:&Self, out:&mut Self::Target);

    fn from_value(version:Option<f32>, value:&Value) -> Result<Self, Cow<'static,str>>;

    fn to_value(&self) -> Value;

    fn version() -> Option<f32> {
        None
    }

    fn anim_type() -> &'static str {
        Self::type_path()
    }
}

#[derive(TypePath)]
pub struct Scale(Vec3);

impl AnimatableValue for Scale {
    type Target = Vec3;

    fn interpolate(&self, s: f32, next: &Self, out: &mut Self::Target) {
        let scale = self.0;
        let next_scale = next.0;
        out.x = scale.x + (next_scale.x - scale.x) * s;
        out.y = scale.y + (next_scale.y - scale.y) * s;
        out.z = scale.z + (next_scale.z - scale.z) * s;
    }

    fn from_value(_version: Option<f32>, value: &Value) -> Result<Self, Cow<'static, str>> {
        let values = value.as_array().ok_or( Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") ) )?;
        if values.len() != 3 {
            return Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( Err( Cow::Borrowed("`Scale`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Err( Cow::Borrowed("`Scale`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Err( Cow::Borrowed("`Scale`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let scale = self.0;
        Value::Array(vec![Value::from(scale.x), Value::from(scale.y), Value::from(scale.z)])
    }
}

#[derive(TypePath)]
pub struct Rotation(Quat);

impl AnimatableValue for Rotation {
    type Target = Quat;

    fn interpolate(&self, s: f32, next: &Self, out: &mut Self::Target) {
        *out = self.0.slerp( next.0, s);
    }

    fn from_value(_version: Option<f32>, value: &Value) -> Result<Self, Cow<'static, str>> {
        let values = value.as_array().ok_or( Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") ) )?;
        if values.len() != 4 {
            return Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") );
        }

        Ok( Self(Quat::from_array([
            values[0].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[3].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[3] is not a number(float)") ) )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let rotation = self.0;
        Value::Array(vec![Value::from(rotation.x), Value::from(rotation.y), Value::from(rotation.z), Value::from(rotation.w) ])
    }
}


#[derive(TypePath)]
pub struct Translation(Vec3);

impl AnimatableValue for Translation {
    type Target = Vec3;

    fn interpolate(&self, s: f32, next: &Self, out: &mut Self::Target) {
        let translation = self.0;
        let next_translation = next.0;
        out.x = translation.x + (next_translation.x - translation.x) * s;
        out.y = translation.y + (next_translation.y - translation.y) * s;
        out.z = translation.z + (next_translation.z - translation.z) * s;
    }

    fn from_value(_version: Option<f32>, value: &Value) -> Result<Self, Cow<'static, str>> {
        let values = value.as_array().ok_or( Err( Cow::Borrowed("`Translation` keyframe must be [f32;3] array") ) )?;
        if values.len() != 3 {
            return Err( Cow::Borrowed("`Translation` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( Err( Cow::Borrowed("`Translation`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Err( Cow::Borrowed("`Translation`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Err( Cow::Borrowed("`Translation`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let translation = self.0;
        Value::Array(vec![Value::from(translation.x), Value::from(translation.y), Value::from(translation.z)])
    }
}