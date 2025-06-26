use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::AssetId;
use bevy_asset::prelude::Asset;
use bevy_asset::uuid::Uuid;
use bevy_reflect::TypePath;
use crate::timeline::Keyframe;
use serde::Deserialize;
use crate::{AnimatableValue, Timeline, TimelineId};

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
    pub name: String,
    pub duration: f32,
    pub targets: Vec<TimelineAnimationTarget>,
}