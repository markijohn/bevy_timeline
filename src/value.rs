use std::any::TypeId;
use std::borrow::Cow;
use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::{ComponentMutability, Mutable};
use bevy_ecs::entity::Entity;
use bevy_reflect::TypePath;
use bevy_ecs::prelude::{Component, Query};
use bevy_ecs::query::QueryData;
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

use serde_json::{json,Value};
use crate::{TimelineError, TimelinePlayer};
use crate::data::{TimelineKeyframe, TimelineUntypedKeyframes};
use crate::player::TimelinePlayback;

pub trait AnimatableValue:Clone+Sized+'static {
    type Target: Component<Mutability=Mutable>;
    
    /// TODO : fast keyframe search from cached(last searched index, last proceed time)
    /// `play_time` : progress play time
    /// `pre_idx` : proceed keyframe index
    /// `keyframes` : all keyframes
    /// `out` : output
    /// return : Some(usize) : changed key frame index, None : keyframe not changed
    fn interpolate_from_keyframe(prev_time:f32, curr_time:f32, keyframes:&[TimelineKeyframe<Self>], mut out:Mut<Self::Target>) {
        if keyframes.is_empty() {
            return;
        }

        let (before,next) = match keyframes.binary_search_by(|probe| probe.time.partial_cmp(&curr_time).unwrap()) {
            Ok(i) => {
                let before = if i > 0 {
                    Some(i - 1) 
                } else { None };
                let next = Some(i);
                (before, next)
            }
            Err(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = if i < keyframes.len() { Some(i) } else { None };
                (before, next)
            }
        };
        let (bef,next) = (before.map(|idx| &keyframes[idx]), next.map(|idx| &keyframes[idx]));

        match (bef, next) {
            (Some(bef), None) => {
                // TODO : If there is no next keyframe to process and the previous keyframe processed is 
                // the same as the start keyframe, no processing is required, i.e., no Mut value is substituted, which prevents bevy from being marked Changed.
                //end of keyframe
                bef.interpolate(0, bef.data.clone(), None, out.as_mut());
            }
            (None, Some(next)) => {
                //no start keyframe
            }
            (Some(bef), Some(next)) => {
                let time_diff = next.time - bef.time;
                let s = (curr_time - bef.time) / time_diff;
                bef.interpolate(s, bef.data.clone(), Some(next.data.clone()), out.as_mut());
            }
            (None, None) => {
                //No frames
            }
        }
    }
    
    fn interpolate(s:f32, start:Self, end:Option<Self>, out:&mut Self::Target);

    fn from_value(value:&Value) -> Result<Self, TimelineError>;

    fn to_value(&self) -> Value;

    fn craete_untyped_keyframes(value:&[Value]) -> Result<TimelineUntypedKeyframes, TimelineError> {
        // let keyframe = value.as_object().ok_or( TimelineError::IncorrectValueType("keyframe is not object") )?;
        // let time = keyframe.get("time").ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not exist") )?.as_f64().ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not number") )? as f32;
        // let data = Self::from_value( keyframe.get("data").ok_or( TimelineError::IncorrectValueType("data(in keyframe) is not exist") )? )?;

        let mut keyframes = Vec::<TimelineKeyframe<Self>>::with_capacity( value.len() );
        for i in value {
            keyframes.push( TimelineKeyframe::from::<Self>(i)? );
        }
        let addr = keyframes.as_mut_ptr() as usize;
        let length = keyframes.len();
        let capacity = keyframes.capacity();
        keyframes.leak();
        let dropper = Box::new( move || {
            unsafe { Vec::from_raw_parts(addr as *mut Self, length, capacity); }
        });
        Ok( TimelineUntypedKeyframes {
            typ: Self::typ(),
            addr,
            length,
            dropper
        })
    }

    fn typ() -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(TypePath)]
pub struct Scale(Vec3);

impl AnimatableValue for Scale {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.scale = (end.0 - start.0) * s;
        } else {
            out.scale = start.0
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
}

impl <T,A,B> AnimatableValue for (A,B) where T:Component<Mutability=Mutable>, A:AnimatableValue<Target=T>+Clone, B:AnimatableValue<Target=T>+Clone {
    type Target = T;

    fn interpolate(s: f32, start: Self, end: Self, out: &mut Self::Target) {

    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
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
            return Err( TimelineError::IncorrectValueType("`Rotation` keyframe must be [f32;3] array") );
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
}


#[derive(TypePath)]
pub struct Translation(Vec3);

impl AnimatableValue for Translation {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.translation = (end.0 - start.0) * s;
        } else {
            out.translation = start.0;
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
}
