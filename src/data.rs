use std::borrow::Cow;
use std::collections::HashMap;
use bevy_app::App;
use bevy_asset::AssetId;
use bevy_asset::prelude::Asset;
use bevy_asset::uuid::Uuid;
use bevy_reflect::TypePath;
use crate::timeline::Keyframe;
use serde::Deserialize;
use serde_json::Value;
use crate::{AnimatableValue, Timeline, TimelineError, TimelineId, TimelineImplPlugin};

type UntypedResolverFn = dyn Fn(&serde_json::Value) -> Result<TimelineUntypedResolvedData, Cow<'static, str>>;

pub struct TimelineUntypedResolver {
    pub typ:&'static str,
    resolver: Box<UntypedResolverFn>
}

struct TimelineUntypedResolvedData {
    typ:&'static str,
    addr: usize,
    len: usize,
    dropper: Box<dyn FnOnce()>
}

impl TimelineUntypedResolver {
    pub fn new<V>( ) -> Self where V:AnimatableValue + Send + Sync + TypePath + 'static {
        Self {
            typ: V::typ(),
            resolver: Box::new( |value| {
                let array = value.as_array().ok_or(Cow::Borrowed("expected array"))?;
                let mut vec = Vec::with_capacity(array.len());
                for v in array {
                    vec.push( V::from_value(&v)? );
                }
                
                let addr = vec.as_mut_ptr() as usize;
                let len = vec.len();
                let capacity = vec.capacity();
                vec.leak();
                let dropper = Box::new( move || {
                    unsafe { Vec::from_raw_parts(addr as *mut V, len, capacity); }
                });
                Ok( TimelineUntypedResolvedData{ typ:V::typ(), addr, len, dropper } )
            })
        }
    }

    pub fn resolve(&self, value:&Value) -> Result<TimelineUntypedResolvedData, Cow<'static, str>> {
        (self.resolver)(value)
    }
}



#[derive(Deserialize)]
struct TimelineAnimationNotResolved {
    pub name: String,
    pub bind_names: Vec<Vec<String>>,
    pub duration: f32,
    pub targets: Vec<TimelineAnimationTargetNotResolved>,
}

#[derive(Deserialize)]
pub struct TimelineAnimationTargetNotResolved {
    pub name: String,
    pub typ: String,
    pub duration: f32,
    pub keyframes: Vec<serde_json::Value>,
}


#[derive(TypePath,Asset)]
pub struct TimelineAnimationSet {
    anims: Vec<TimelineUntypedAnimation>
}

impl TimelineAnimationSet {
    pub fn from(resolver:HashMap<&'static str,TimelineUntypedResolver>, value:&Value) -> Self {
        serde_json::from_value( value );
    }
}

#[derive(TypePath,Asset)]
pub struct TimelineUntypedAnimation {
    pub name: String,
    pub bind_names: Vec<Vec<String>>,
    pub duration: f32,
    pub targets: Vec<TimelineUntypedTarget>,
}

pub struct TimelineKeyframe<T> {
    pub time:f32,
    pub data:T
}


#[derive(TypePath,Asset)]
pub struct TimelineUntypedTarget {
    pub bind_idx: usize,
    pub typ: &'static str,
    addr: usize,
    length: usize,
    dropper : Box<dyn Fn() + Send + Sync + 'static>,
}


impl TimelineUntypedTarget {
    pub fn from(bind_idx:usize, type_resolver:&TimelineUntypedResolver, value:&Value) -> Result<Self, Cow<'static,str>> {
        let TimelineUntypedResolvedData{typ, addr,len,dropper, .. } = type_resolver.resolve( &value )?;
        Ok( Self {
            bind_idx,
            typ,
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