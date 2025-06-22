use std::collections::HashMap;
use bevy_asset::prelude::Asset;
use bevy_reflect::TypePath;
use crate::timeline::Keyframe;
use serde::Deserialize;

#[derive(Deserialize,TypePath,Asset)]
pub struct TimelineRawData {
    pub anims: HashMap<String, TimelineAnimation>
}

#[derive(Deserialize)]
pub struct TimelineAnimationTarget {
    pub name: String,
    pub keyframes: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct TimelineAnimation {
    pub duration: f32,
    pub keyframes: Vec<TimelineAnimationTarget>,
}