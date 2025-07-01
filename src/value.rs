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
    fn interpolate(s:f32, start:Self, end:Self, out:&mut Self::Target);

    fn from_value(value:&Value) -> Result<Self, Cow<'static,str>>;

    fn to_value(&self) -> Value;

    fn typ() -> &'static str {
        Self::type_path()
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

pub trait AnimatableSet {
    fn build(app:&mut bevy_app::App);


}

macro_rules! impl_animatable_set {
    // 2개 요소 튜플
    ($($T:ident),+ $(,)?) => {
        impl_animatable_list!(@impl $($T),+);
    };

    // 실제 구현 생성
    (@impl $($T:ident),+) => {
        impl<$($T),+> AnimatableSet for ($($T,)+)
        where
            $($T: Animatable,)+
        {
            fn build(app:&mut bevy_app::App) {
                app.
            }

            fn animate_all(&self) {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                $(
                    $T.animate();
                )+
            }
        }
    };
}

macro_rules! impl_animatable_set_recursive {
    () => {};

    ($head:ident $(, $tail:ident)*) => {
        impl_animatable_set!($head $(, $tail)*);
        impl_animatable_set_recursive!($($tail),*);
    };
}

impl_animatable_list_recursive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
