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
use crate::{AnimatableValue, Timeline, TimelineId, TimelineImplPlugin};

type UntypedResolverFn = dyn Fn(&serde_json::Value) -> Result<(Box<dyn FnOnce()>, usize, usize, usize), Cow<'static, str>>;

pub struct TimelineUntypedResolver {
    typ:&'static str,
    plugin_registar: Box<dyn Fn(&mut App)>,
    pub resolver: Box<UntypedResolverFn>
}

impl TimelineUntypedResolver {
    pub fn new<V>( ) -> Self where V:AnimatableValue + Send + Sync + TypePath + 'static {
        Self {
            typ: V::typ(),
            plugin_registar: Box::new( |app:&mut App| {
                app.add_plugins( TimelineImplPlugin::<V>::default() );
            }),
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
                Ok( ( dropper, addr, len, capacity ) )
            })
        }
    }

    pub fn resolve(&self, value:&Value) -> Result<(Box<dyn FnOnce()>, usize, usize, usize), Cow<'static, str>> {
        (self.resolver)(value)
    }
}

#[derive(Deserialize)]
pub struct TimelineAnimationTarget {
    pub name: String,
    pub typ: String,
    pub duration: f32,
    pub keyframes: Vec<serde_json::Value>,
}

impl TimelineAnimationTarget {
    pub fn to_timeilne<K>(&self) -> Result<Timeline<K>, Cow<'static, str>> 
        where K:AnimatableValue + Send + Sync {
        let keyframes = Keyframe::<K>::load_frames(&self.keyframes)?;
        Ok( Timeline::new(self.name.clone(), self.duration, keyframes) )
    }
}

#[derive(Deserialize)]
struct TimelineAnimationNotResolved {
    pub name: String,
    pub bind_names: Vec<Vec<String>>,
    pub duration: f32,
    pub targets: Vec<TimelineAnimationTarget>,
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



pub struct TimelineUntypedTarget {
    pub bind_idx: usize,
    pub typ: &'static str,
    addr: usize,
    length: usize,
    capacity: usize,
    dropper : Box<dyn Fn() + Send + Sync + 'static>,
}


impl TimelineUntypedTarget {
    pub fn from(bind_idx:usize, type_resolver:&TimelineUntypedResolver, value:&Value) -> Self {
        let (typ, addr,length,capacity,dropper) = type_resolver.resolve( &value );
        Self {
            bind_idx,
            typ,
            addr,
            length,
            capacity,
            dropper
        }
    }
    pub fn get_typed<T>(&self) -> &[TimelineKeyframe<T>] {
        unsafe {
            std::slice::from_raw_parts(self.addr as *const TimelineKeyframe<T>, self.length)
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