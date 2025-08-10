use std::ops::Deref;
use bevy_math::Curve;
use bevy_math::prelude::{EaseFunction};
use bevy_transform::prelude::Transform;
use bevy_timeline_runtime::prelude::{AnimatableValue};
use serde::{Serialize,Deserialize};

pub type TransformSet = (Scale, Rotation, Translation);

#[derive(Clone,Serialize,Deserialize)]
pub struct EaseFunctionWrap<V> {
    ease : EaseFunction,
    value : V
}

impl <V> Default for EaseFunctionWrap<V> where V:AnimatableValue {
    fn default() -> Self {
        Self {
            ease: EaseFunction::Linear,
            value: Default::default()
        }
    }
}

impl <V> AnimatableValue for EaseFunctionWrap<V> where V:AnimatableValue {
    type Target = V::Target;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(v) = end {
            V::interpolate(start.ease.sample_unchecked(s), start.value, Some(v.value), out);
        } else {
            V::interpolate(s, start.value, None, out);
        }
    }

    fn typ() -> &'static str {
        V::typ()
    }
}

#[derive(Clone,Default,Serialize,Deserialize)]
pub struct _RotaTrans {
    rot : bevy_timeline_runtime::value::Rotation,
    trans : bevy_timeline_runtime::value::Translation,
}

pub type Scale = EaseFunctionWrap<bevy_timeline_runtime::value::Scale>;
type Rotation = EaseFunctionWrap<bevy_timeline_runtime::value::Rotation>;
type Translation = EaseFunctionWrap<bevy_timeline_runtime::value::Translation>;

impl AnimatableValue for _RotaTrans {
    type Target = Transform;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(v) = end {
            bevy_timeline_runtime::value::Rotation::interpolate(s, start.rot, Some(v.rot), out);
            bevy_timeline_runtime::value::Translation::interpolate(s, start.trans, Some(v.trans), out);
        } else {
            bevy_timeline_runtime::value::Rotation::interpolate(s, start.rot, None, out);
            bevy_timeline_runtime::value::Translation::interpolate(s, start.trans, None, out);
        }
    }
}

pub type RotaTrans = EaseFunctionWrap<_RotaTrans>;