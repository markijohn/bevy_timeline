use std::any::TypeId;
use std::borrow::Cow;
use bevy_ecs::component::{ComponentMutability, Mutable};
use bevy_ecs::entity::Entity;
use bevy_reflect::TypePath;
use bevy_ecs::prelude::{Component, Query};
use bevy_ecs::query::QueryData;
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

use serde_json::{json,Value};
use crate::data::TimelineUntypedAnimation;
use crate::TimelinePlayer;

pub trait AnimatableValue:Sized+'static {
    type Target: Component<Mutability=Mutable>;
    fn interpolate(s:f32, start:Self, end:Self, out:&mut Self::Target);

    fn from_value(value:&Value) -> Result<Self, Cow<'static,str>>;

    fn to_value(&self) -> Value;

    fn typ() -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(TypePath)]
pub struct Scale(Vec3);

impl AnimatableValue for Scale {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Self, out: &mut Self::Target) {
        out.scale = (end.0 - start.0) * s;
    }

    fn from_value(value: &Value) -> Result<Self, Cow<'static, str>> {
        let values = value.as_array().ok_or( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") )?;
        if values.len() != 3 {
            return Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( Cow::Borrowed("`Scale`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Cow::Borrowed("`Scale`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Cow::Borrowed("`Scale`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let scale = self.0;
        Value::Array(vec![Value::from(scale.x), Value::from(scale.y), Value::from(scale.z)])
    }
}

impl <T,A,B> AnimatableValue for (A,B) where T:Component<Mutability=Mutable>, A:AnimatableValue<Target=T>+Clone, B:AnimatableValue<Target=T>+Clone {
    type Target = T;

    fn interpolate(s: f32, start: Self, end: Self, out: &mut Self::Target) {
        
    }

    fn from_value(value: &Value) -> Result<Self, Cow<'static, str>> {
        todo!()
    }

    fn to_value(&self) -> Value {
        todo!()
    }
}

#[derive(TypePath)]
pub struct Rotation(Quat);

impl AnimatableValue for Rotation {
    type Target = Transform;

    fn interpolate(s: f32, prev:Self, next: Self, out: &mut Self::Target) {
        out.rotation = prev.0.slerp( next.0, s );
    }

    fn from_value(value: &Value) -> Result<Self, Cow<'static, str>> {
        let values = value.as_array().ok_or( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") )?;
        if values.len() != 4 {
            return Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") );
        }

        Ok( Self(Quat::from_array([
            values[0].as_number().ok_or( Cow::Borrowed("`Rotation`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Cow::Borrowed("`Rotation`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Cow::Borrowed("`Rotation`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[3].as_number().ok_or( Cow::Borrowed("`Rotation`[3] is not a number(float)") )?.as_f64().unwrap() as f32,
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
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Self, out: &mut Self::Target) {
        out.translation = (end.0 - start.0) * s;
    }

    fn from_value(value: &Value) -> Result<Self, Cow<'static, str>> {
        let values = value.as_array().ok_or( Cow::Borrowed("`Translation` keyframe must be [f32;3] array") )?;
        if values.len() != 3 {
            return Err( Cow::Borrowed("`Translation` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( Cow::Borrowed("`Translation`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Cow::Borrowed("`Translation`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Cow::Borrowed("`Translation`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let translation = self.0;
        Value::Array(vec![Value::from(translation.x), Value::from(translation.y), Value::from(translation.z)])
    }
}
