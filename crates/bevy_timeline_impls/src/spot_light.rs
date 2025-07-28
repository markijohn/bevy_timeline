use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde::{Serialize,Deserialize};

use bevy_pbr::SpotLight;

pub type SpotLightSet = (SpotLightColor, SpotLightIntensity, SpotLightRange, SpotLightRadius);

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SpotLightColor(Color);

impl AnimatableValue for SpotLightColor {
    type Target = SpotLight;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.color = start.0.mix( &end.0, s );
        } else {
            out.color = start.0;
        }
    }
}


#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SpotLightIntensity(f32);

impl AnimatableValue for SpotLightIntensity {
    type Target = SpotLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.intensity = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SpotLightRange(f32);

impl AnimatableValue for SpotLightRange {
    type Target = SpotLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.range = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct SpotLightRadius(f32);

impl AnimatableValue for SpotLightRadius {
    type Target = SpotLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.radius = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}