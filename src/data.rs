use std::borrow::Cow;
use std::collections::HashMap;
use bevy_app::App;
use bevy_asset::{AssetId, Handle};
use bevy_asset::prelude::Asset;
use bevy_asset::uuid::Uuid;
use bevy_reflect::TypePath;
use crate::timeline::Keyframe;
use serde::Deserialize;
use serde_json::Value;
use crate::{AnimatableValue, Timeline, TimelineError, TimelineImplSets};


#[derive(TypePath,Asset)]
pub struct TimelineAnimationSet(pub(crate) Vec<Handle<TimelineAnimation>>);

#[derive(TypePath,Asset)]
pub struct TimelineAnimation {
    pub name: String,
    pub duration: f32,
    pub targets: Vec<TimelineUntypedTarget>,
}

impl TimelineAnimation {
    pub fn load_animation<V:TimelineImplSets>(value:&Value) -> Result<TimelineAnimation, TimelineError> {
        let map = value.as_object().ok_or(TimelineError::IncorrectValueType("animation is not an object"))?;
        let name = map.get("name").ok_or(TimelineError::IncorrectValueType("name(in animation) is not exist"))?.as_str().ok_or(TimelineError::IncorrectValueType("timeline name must be string"))?.to_string();
        let duration = map.get("name").ok_or(TimelineError::IncorrectValueType("duration(in animation) is not exist"))?.as_number().ok_or(TimelineError::IncorrectValueType("duration must be number"))?.as_f64().unwrap() as f32;
        let targets_value = map.get("targets").ok_or(TimelineError::IncorrectValueType("targets(in animation) is not exist"))?.as_array().ok_or(TimelineError::IncorrectValueType("targets must be array"))?;
        let mut targets = Vec::with_capacity(targets_value.len());
        for i in targets_value {
            targets.push( TimelineUntypedTarget::from::<V>( i )? );
        }
        Ok(TimelineAnimation { name, duration, targets })
    }
    pub fn load_animations<V:TimelineImplSets>(value:&Value) -> Result<Vec<TimelineAnimation>, TimelineError> {
        let anims_value = value.as_array().ok_or("timeline must be array")?;
        let mut anims = Vec::with_capacity( anims_value.len() );
        for i in anims_value {
            anims.push( Self::load_animation::<V>( i )? );
        }
        Ok( anims )
    }
}

pub struct TimelineKeyframe<T> {
    pub time:f32,
    pub data:T
}

impl <T> TimelineKeyframe<T> where T:AnimatableValue {
    pub fn from<V:AnimatableValue>(value:&Value) -> Result<TimelineKeyframe<T>, TimelineError>{
        let keyframe = value.as_object().ok_or( TimelineError::IncorrectValueType("keyframe is not object") )?;
        let time = keyframe.get("time").ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not exist") )?.as_f64().ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not number") )? as f32;
        let data = V::from_value( keyframe.get("data").ok_or( TimelineError::IncorrectValueType("data(in keyframe) is not exist") )? )?;
        Ok( TimelineKeyframe { time,data } )
    }
}

#[derive(TypePath,Asset)]
pub struct TimelineUntypedTarget {
    pub typ: &'static str,
    target: Vec<String>,
    addr: usize,
    length: usize,
    dropper : Box<dyn Fn() + Send + Sync + 'static>,
}


impl TimelineUntypedTarget {
    pub fn from<V:AnimatableValue>(value:&Value) -> Result<Self, TimelineError> {
        let map = value.as_object().ok_or( TimelineError::IncorrectValueType("target must be object") )?;
        let target_value = map.get("target").ok_or(TimelineError::IncorrectValueType("target not exist"))?
            .as_array().ok_or(TimelineError::IncorrectValueType("target must be string array"))?;
        let mut target = Vec::with_capacity(target_value.len());
        for i in target_value {
            target.push( value.as_str().ok_or(TimelineError::IncorrectValueType("target must be string"))?.to_string() );
        }
        let value = map.get("keyframes").ok_or(TimelineError::IncorrectValueType("keyframes not exist"))?
            .as_array().ok_or(TimelineError::IncorrectValueType("keyframes is not array"))?;
        let mut keyframes = Vec::<TimelineKeyframe<V>>::with_capacity( value.len() );
        for i in value {
            keyframes.push( TimelineKeyframe::<V>::from(i)? );
        }
        let addr = keyframes.as_mut_ptr() as usize;
        let length = keyframes.len();
        let capacity = keyframes.capacity();
        keyframes.leak();
        let dropper = Box::new( move || {
            unsafe { Vec::from_raw_parts(addr as *mut V, length, capacity); }
        });
        Ok( Self {
            target,
            typ: V::typ(),
            addr,
            length,
            dropper
        } )
    }

    pub fn get_typed<T:AnimatableValue>(&self) -> Result<&[TimelineKeyframe<T>], TimelineError> {
        unsafe {
            if T::typ() == self.typ {
                Ok( std::slice::from_raw_parts(self.addr as *const TimelineKeyframe<T>, self.length) )
            } else {
                Err( TimelineError::TypeNotMatch {request:T::typ(), actual: self.typ})
            }
        }
    }
}

impl Drop for TimelineUntypedTarget {
    fn drop(&mut self) {
        unsafe {
            (self.dropper) ();
        }
    }
}