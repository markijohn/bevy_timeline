use std::borrow::Cow;
use bevy_ecs::prelude::Component;
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

#[cfg(feature="json_serialize")]
use serde_json::Value;

pub trait AnimatableValue {
    type Target: Component;
    fn interpolate(&self, s:f32, next:&Self, out:&mut Self::Target);

    #[cfg(feature="json_serialize")]
    fn from_value(value:&Value) -> Result<Self, Cow<'static,str>>;
}

pub struct Trans( Vec3 );

impl AnimatableValue for Trans {
    type Target = Transform;
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        let mut out = &mut out.translation;
        out.x = self.0.x + (next.0.x - self.0.x) * s;
        out.y = self.0.y + (next.0.y - self.0.y) * s;
        out.z = self.0.z + (next.0.z - self.0.z) * s;
    }

    #[cfg(feature="json_serialize")]
    fn from_value(value:&Value) -> Result<Self, Cow<'static,str>> {
        let values = value.as_array().ok_or( Err( Cow::Borrowed("`Trans` keyframe must be [f32;3] array") ) )?;
        if values.len() != 3 {
            return Err( Cow::Borrowed("`Trans` keyframe must be [f32;3] array") );
        }
        let r = Self(Vec3::from_array([
            values[0].as_number().ok_or( Err( Cow::Borrowed("`Trans`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Err( Cow::Borrowed("`Trans`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Err( Cow::Borrowed("`Trans`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
        ]));
        Ok( r )
    }
}

pub struct Scale( Vec3 );

impl AnimatableValue for Scale {
    type Target = Transform;
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        let mut out = &mut out.scale;
        out.x = self.0.x + (next.0.x - self.0.x) * s;
        out.y = self.0.y + (next.0.y - self.0.y) * s;
        out.z = self.0.z + (next.0.z - self.0.z) * s;
    }

    #[cfg(feature="json_serialize")]
    fn from_value(value:&Value) -> Result<Self, Cow<'static,str>> {
        let values = value.as_array().ok_or( Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") ) )?;
        if values.len() != 3 {
            return Err( Cow::Borrowed("`Scale` keyframe must be [f32;3] array") );
        }
        let r = Self(Vec3::from_array([
            values[0].as_number().ok_or( Err( Cow::Borrowed("`Scale`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Err( Cow::Borrowed("`Scale`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Err( Cow::Borrowed("`Scale`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
        ]));
        Ok( r )
    }
}

pub struct Rotation( Quat );
impl AnimatableValue for Rotation {
    type Target = Transform;
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        let mut out = &mut out.rotation;
        *out = self.0.slerp(next.0, s);
    }

    #[cfg(feature="json_serialize")]
    fn from_value(value:&Value) -> Result<Self, Cow<'static,str>> {
        let values = value.as_array().ok_or( Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") ) )?;
        if values.len() != 4 {
            return Err( Cow::Borrowed("`Rotation` keyframe must be [f32;3] array") );
        }

        let r = Self(Quat::from_array([
            values[0].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[0] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[1] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[2] is not a number(float)") ) )?.as_f64().unwrap() as f32,
            values[3].as_number().ok_or( Err( Cow::Borrowed("`Rotation`[3] is not a number(float)") ) )?.as_f64().unwrap() as f32,
        ]));
        Ok( r )
    }
}

pub struct TLTransform {
    pub scale: Option<Scale>,
    pub rotation: Option<Rotation>,
    pub translation: Option<Trans>,
}

impl AnimatableValue for TLTransform {
    type Target = Transform;
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        if let (Some(scale), Some(next_scale)) = (self.scale.as_ref(), next.scale.as_ref()) {
            scale.interpolate(s, next_scale, out);
        }
        if let (Some(rot), Some(next_rot)) = (self.rotation.as_ref(), next.rotation.as_ref()) {
            rot.interpolate(s, next_rot, out);
        }
        if let (Some(translation), Some(next_translation)) = (self.translation.as_ref(), next.translation.as_ref()) {
            translation.interpolate(s, next_translation, out);
        }
    }

    #[cfg(feature="json_serialize")]
    fn from_value(value: &Value) -> Result<Self, Cow<'static, str>> {
        let map = value.as_object().ok_or( Cow::Borrowed("`TLTransform` must be object") )?;
        Ok( Self {
            scale : map.get("scale").map(|v| Scale::from_value(v).ok() ).unwrap_or( None ),
            rotation : map.get("rotation").map(|v| Rotation::from_value(v).ok() ).unwrap_or( None ),
            translation : map.get("translation").map(|v| Trans::from_value(v).ok() ).unwrap_or( None ),
        })
    }
}