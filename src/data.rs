use std::collections::HashMap;
use bevy_asset::AssetId;
use bevy_asset::prelude::Asset;
use bevy_reflect::TypePath;
use crate::timeline::Keyframe;
use serde::Deserialize;
use crate::TimelineId;

#[derive(Deserialize,TypePath,Asset)]
pub struct TimelineRawData {
    pub anims: Vec<TimelineAnimation>
}

impl TimelineRawData {

}

#[derive(Deserialize)]
pub struct TimelineAnimationTarget {
    pub name: String,
    pub keyframes: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct TimelineAnimation {
    pub name: String,
    pub duration: f32,
    pub targets: Vec<TimelineAnimationTarget>,
}