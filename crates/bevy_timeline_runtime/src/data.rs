use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use bevy_app::App;
use bevy_asset::{AssetId, Handle};
use bevy_asset::prelude::Asset;
use bevy_asset::uuid::Uuid;
use bevy_reflect::TypePath;
use serde::Deserialize;
use serde_json::Value;
use crate::{AnimatableValue, TimelineError, TimelineImplSets};
use crate::value::ValueExt;

#[derive(Debug,TypePath,Asset)]
pub struct TimelineAnimationSet{
    pub path: Option<std::path::PathBuf>,
    pub anim_handles: Vec<Handle<TimelineAnimation>>
}

#[derive(Debug,TypePath,Asset)]
pub struct TimelineAnimation {
    pub name: String,
    pub duration: f32,
    pub targets: Vec<TimelineTarget>,
}

impl TimelineAnimation {
    pub fn load_animation<V:TimelineImplSets>(value:&Value) -> Result<TimelineAnimation, TimelineError> {
        let map = value.as_object().ok_or(TimelineError::IncorrectValueType("animation is not an object"))?;
        let name = map.get("name").ok_or(TimelineError::IncorrectValueType("name(in animation) is not exist"))?.as_str().ok_or(TimelineError::IncorrectValueType("timeline name must be string"))?.to_string();
        let duration = map.get("duration").ok_or(TimelineError::IncorrectValueType("duration(in animation) is not exist"))?.as_number().ok_or(TimelineError::IncorrectValueType("duration must be number"))?.as_f64().unwrap() as f32;
        let targets_value = map.get("targets").ok_or(TimelineError::IncorrectValueType("targets(in animation) is not exist"))?.as_array().ok_or(TimelineError::IncorrectValueType("targets must be array"))?;
        let mut targets = Vec::with_capacity(targets_value.len());
        for i in targets_value {
            targets.push( TimelineTarget::from::<V>( i )? );
        }
        Ok(TimelineAnimation { name, duration, targets })
    }

    pub fn load_animations<V:TimelineImplSets>(value:&Value) -> Result<Vec<TimelineAnimation>, TimelineError> {
        let anims_value = value.as_array().ok_or(TimelineError::IncorrectValueType("timeline must be array") )?;
        let mut anims = Vec::with_capacity( anims_value.len() );
        for i in anims_value {
            anims.push( Self::load_animation::<V>( i )? );
        }
        Ok( anims )
    }
}

// #[derive(Clone)]
// pub struct TimelineKeyframe<T> where T:AnimatableValue{
//     pub time:f32,
//     pub data:T
// }
// 
// impl <T> TimelineKeyframe<T> where T:AnimatableValue {
//     pub fn from(value:&Value) -> Result<TimelineKeyframe<T>, TimelineError>{
//         let keyframe = value.as_object().ok_or( TimelineError::IncorrectValueType("keyframe is not object") )?;
//         let time = keyframe.get("time").ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not exist") )?.as_f64().ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not number") )? as f32;
//         let data = serde_json::from_value::<T>( keyframe["data"].clone() )?;
//         Ok( TimelineKeyframe { time,data } )
//     }
// }

pub struct TimelineUntypedSeq {
    pub typ:&'static str,
    pub(crate) addr: usize,
    pub(crate) length: usize,
    pub(crate) dropper : Box<dyn Fn() + Send + Sync + 'static>,
}

impl TimelineUntypedSeq {
    fn get_typed<T:AnimatableValue>(&self) -> Result<&[T], TimelineError> {
        unsafe {
            if T::typ() == self.typ {
                Ok( std::slice::from_raw_parts(self.addr as *const T, self.length) )
            } else {
                Err( TimelineError::TypeNotMatch {request:T::typ(), actual: self.typ})
            }
        }
    }
}

impl Drop for TimelineUntypedSeq {
    fn drop(&mut self) {
        unsafe {
            (self.dropper) ();
        }
    }
}

#[derive(TypePath,Asset)]
pub struct TimelineTarget {
    pub target: Vec<String>,

    //not serialized data
    pub selected: Vec<bool>,
    
    pub times: Vec<f32>,
    pub seq: TimelineUntypedSeq,
}

impl Debug for TimelineTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimelineUntypedTarget({}, keyframes:{})", self.target.join(","), self.seq.length)
    }
}


impl TimelineTarget {
    pub fn typ(&self) -> &'static str {
        self.seq.typ
    }

    pub fn from<V:TimelineImplSets>(value:&Value) -> Result<Self, TimelineError> {
        let map = value.as_object().ok_or( TimelineError::IncorrectValueType("target must be object") )?;
        let typ = map.get("typ").ok_or(TimelineError::IncorrectValueType("typ not exist"))?
            .as_str().ok_or(TimelineError::IncorrectValueType("typ must be string"))?;
        let target_value = map.get("target").ok_or(TimelineError::IncorrectValueType("target not exist"))?
            .as_array().ok_or(TimelineError::IncorrectValueType("target must be string array"))?;
        let mut target = Vec::with_capacity(target_value.len());
        for i in target_value {
            target.push( i.as_str().ok_or(TimelineError::IncorrectValueType("target must be string"))?.to_string() );
        }
        
        let times = map.get("times").ok_or(TimelineError::IncorrectValueType("times not exist"))?;
        let times = times.as_array_f32()?;
        
        let selected = vec![false; times.len()];
        
        let seq = map.get("seq").ok_or(TimelineError::IncorrectValueType("keyframes not exist"))?
            .as_array().ok_or(TimelineError::IncorrectValueType("keyframes is not array"))?;
        let seq = if let Some(result) = V::try_resolve_keyframes( typ, seq.as_slice() ) {
            result?
        } else {
            return Err(TimelineError::UnknownTargetType(typ.to_string()))
        };
        Ok( Self {
            target,
            selected,
            times,
            seq,
        } )
    }
    
    pub fn get_times(&self) -> &[f32] {
        self.times.as_slice()
    }
    
    pub fn get_times_with_selected(&mut self) -> impl Iterator<Item=(&mut f32,&mut bool)> {
        self.times.iter_mut().zip(self.selected.iter_mut())
    }

    pub fn get_typed<T:AnimatableValue>(&self) -> Result<&[T], TimelineError> {
        self.seq.get_typed::<T>()
    }
}
