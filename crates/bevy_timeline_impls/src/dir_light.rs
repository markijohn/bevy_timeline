use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde_json::{Value};

use bevy_pbr::{DirectionalLight};

pub type DirLightSet = (DirLightColor, DirLightLumen);

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
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

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self(
            serde_json::from_value( value["ease"].clone() )?
        ) )
    }

    fn to_value(&self) -> Value {
        serde_json::to_value( self.0 ).unwrap()
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirLightLumen(f32);


impl AnimatableValue for DirLightLumen {
    type Target = DirectionalLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.illuminance = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self( value.as_f32()? ) )
    }

    fn to_value(&self) -> Value {
        Value::from( self.0 )
    }
}