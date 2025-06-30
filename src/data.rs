use std::borrow::Cow;
use std::collections::HashMap;
use bevy_app::App;
use bevy_asset::AssetId;
use bevy_asset::prelude::Asset;
use bevy_asset::uuid::Uuid;
use bevy_reflect::TypePath;
use crate::timeline::Keyframe;
use serde::Deserialize;
use crate::{AnimatableValue, Timeline, TimelineId, TimelineImplPlugin};

type UntypedResolverFn = dyn Fn(&serde_json::Value) -> (Box<dyn FnOnce()>, usize, usize, usize);

pub struct TimelineUntypedResolver {
    typ:&'static str,
    plugin_registar: Box<dyn Fn(&mut App)>,
    resolve: Box<UntypedResolverFn>
}

impl TimelineUntypedResolver {
    pub fn new<V:AnimatableValue>( ) -> Self {
        Self {
            typ: V::typ(),
            plugin_registar: Box::new( |app:&mut App| {
                app.add_plugins( TimelineImplPlugin::<V>::default() );
            }),
            resolve: Box::new( |value| {
                todo!()
            })
        }
    }
}


#[derive(Deserialize,TypePath,Asset)]
pub struct TimelineRawData {
    pub anims: Vec<TimelineAnimation>
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
pub struct TimelineAnimation {
    pub bind_names: Vec<Vec<String>>,
    pub name: String,
    pub duration: f32,
    pub targets: Vec<TimelineAnimationTarget>,
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
    dropper : Box<dyn FnOnce()>,
}


impl TimelineUntypedTarget {
    pub fn new(type_resolver:&TimelineUntypedResolver, value:&Value) {
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
            std::slice::from_raw_parts(self.addr as *const T, self.length)
        }
    }
}

impl Drop for TimelineUntypedTarget {
    fn drop(&mut self) {
        unsafe {
            self.dropper();
        }
    }
}