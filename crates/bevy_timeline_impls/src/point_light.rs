use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use bevy_timeline_runtime::prelude::{ValueExt, AnimatableValue, TimelineError};
use serde_json::{Value};

use bevy_pbr::PointLight;

pub type PointLightSet = (PointLightColor, PointLightIntensity, PointLightRadius);

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
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
pub struct PointLightIntensity(f32);

impl AnimatableValue for PointLightIntensity {
    type Target = PointLight;

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
pub struct PointLightRadius(f32);

impl AnimatableValue for PointLightRadius {
    type Target = PointLight;

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