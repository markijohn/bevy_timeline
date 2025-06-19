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
        Self::type_ident().unwrap()
    }
}

#[derive(TypePath)]
pub struct TLTransform {
    pub scale: Option<Vec3>,
    pub rotation: Option<Quat>,
    pub translation: Option<Vec3>,
}

impl AnimatableValue for TLTransform {
    type Target = Transform;
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        if let (Some(scale), Some(next_scale)) = (self.scale.as_ref(), next.scale.as_ref()) {
            let mut out = &mut out.scale;
            out.x = scale.x + (next_scale.x - scale.x) * s;
            out.y = scale.y + (next_scale.y - scale.y) * s;
            out.z = scale.z + (next_scale.z - scale.z) * s;
        }
        if let (Some(rot), Some(next_rot)) = (self.rotation.as_ref(), next.rotation.as_ref()) {
            let mut out = &mut out.rotation;
            *out = rot.slerp(*next_rot, s);
        }
        if let (Some(translation), Some(next_translation)) = (self.translation.as_ref(), next.translation.as_ref()) {
            let mut out = &mut out.translation;
            out.x = translation.x + (next_translation.x - translation.x) * s;
            out.y = translation.y + (next_translation.y - translation.y) * s;
            out.z = translation.z + (next_translation.z - translation.z) * s;
        }
    }

    fn from_value(_version:Option<f32>, value: &Value) -> Result<Self, Cow<'static, str>> {
        let scale = if let Some(values) = value.get("scale") {
            let values = values.as_array().ok_or( Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") ) )?;
            if values.len() != 3 {
                return Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") );
            }
            Some(Vec3::from_array([
                values[0].as_number().ok_or( Err( Cow::Borrowed("`Scale`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[1].as_number().ok_or( Err( Cow::Borrowed("`Scale`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[2].as_number().ok_or( Err( Cow::Borrowed("`Scale`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            ]))
        } else { None };

        let rotation = if let Some(values) = value.get("rotation") {
            let values = values.as_array().ok_or( Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") ) )?;
            if values.len() != 4 {
                return Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") );
            }

            Some(Quat::from_array([
                values[0].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[1].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[2].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[3].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[3] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            ]))
        } else { None };

        let translation = if let Some(values) = value.get("translation") {
            let values = values.as_array().ok_or( Err( Cow::Borrowed("`Trans` keyframe must be [f32;3] array") ) )?;
            if values.len() != 3 {
                return Err( Cow::Borrowed("`Trans` keyframe must be [f32;3] array") );
            }
            Some(Vec3::from_array([
                values[0].as_number().ok_or( Err( Cow::Borrowed("`Trans`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[1].as_number().ok_or( Err( Cow::Borrowed("`Trans`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
                values[2].as_number().ok_or( Err( Cow::Borrowed("`Trans`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            ]))
        } else { None };

        Ok( Self {
            scale,
            rotation,
            translation,
        })
    }

    fn to_value(&self) -> Value {
        let scale = if let Some(scale) = self.scale {
            Some(Value::Array(vec![Value::from(scale.x), Value::from(scale.y), Value::from(scale.z)]))
        } else { None };
        let rotation = if let Some(rotation) = self.rotation {
            Some(Value::Array(vec![Value::from(rotation.x), Value::from(rotation.y), Value::from(rotation.z), Value::from(rotation.w) ]))
        } else { None };
        let translation = if let Some(translation) = self.scale {
            Some(Value::Array(vec![Value::from(translation.x), Value::from(translation.y), Value::from(translation.z)]))
        } else { None };

        json!( {
            "scale" : scale,
            "rotation" : rotation,
            "translation" : translation
        } )
    }
}