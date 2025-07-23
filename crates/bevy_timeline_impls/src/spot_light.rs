use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde_json::{Value};

use bevy_pbr::SpotLight;

pub type SpotLightSet = (SpotLightColor, SpotLightIntensity, SpotLightRange, SpotLightRadius);

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
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

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self(
            serde_json::from_value( value.clone() )?
        ) )
    }

    fn to_value(&self) -> Value {
        serde_json::to_value( self.0 ).unwrap()
    }
}


#[derive(Default, Clone)]
pub struct SpotLightIntensity(f32);

impl AnimatableValue for SpotLightIntensity {
    type Target = SpotLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.intensity = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self( value.as_f32()? ) )
    }

    fn to_value(&self) -> Value {
        Value::from( self.0 )
    }
}

#[derive(Default, Clone)]
pub struct SpotLightRange(f32);

impl AnimatableValue for SpotLightRange {
    type Target = SpotLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.range = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self( value.as_f32()? ) )
    }

    fn to_value(&self) -> Value {
        Value::from( self.0 )
    }
}

#[derive(Default, Clone)]
pub struct SpotLightRadius(f32);

impl AnimatableValue for SpotLightRadius {
    type Target = SpotLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.radius = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self( value.as_f64().ok_or(TimelineError::IncorrectValueType("PointLightRadius must be f32"))? as _ ) )
    }

    fn to_value(&self) -> Value {
        Value::from( self.0 )
    }
}