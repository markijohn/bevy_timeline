use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde::{Serialize, Deserialize};

use bevy_pbr::PointLight;

pub type PointLightSet = (PointLightColor, PointLightIntensity, PointLightRadius);

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PointLightColor(Color);

impl AnimatableValue for PointLightColor {
    type Target = PointLight;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.color = start.0.mix( &end.0, s );
        } else {
            out.color = start.0;
        }
    }
}


#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PointLightIntensity(f32);

impl AnimatableValue for PointLightIntensity {
    type Target = PointLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.intensity = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PointLightRadius(f32);

impl AnimatableValue for PointLightRadius {
    type Target = PointLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.radius = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}