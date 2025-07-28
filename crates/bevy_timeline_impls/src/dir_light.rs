use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde::{Serialize,Deserialize};

use bevy_pbr::{DirectionalLight};

pub type DirLightSet = (DirLightColor, DirLightLumen);

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DirLightColor(Color);

impl AnimatableValue for DirLightColor {
    type Target = DirectionalLight;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.color = start.0.mix( &end.0, s );
        } else {
            out.color = start.0;
        }
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirLightLumen(f32);


impl AnimatableValue for DirLightLumen {
    type Target = DirectionalLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.illuminance = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}