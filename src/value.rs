use bevy_ecs::prelude::Component;
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

pub trait AnimatableValue<Out> {
    fn interpolate(&self, s:f32, next:&Self, out:&mut Out);
}


// pub trait AnimatableValue<Out=Self> {
//     fn tl_lerp(&self, s:f32, next:&Self, out:&mut Out);
// }
//
// impl AnimatableValue for f32 {
//     fn tl_lerp(&self, s:f32, next:&f32, out:&mut f32) {
//         *out = self + (next - self) * s;
//     }
// }
//
// impl AnimatableValue for Vec3 {
//     fn tl_lerp(&self, s: f32, next: &Self, out: &mut Self) {
//         out.x = self.x + (next.x - self.x) * s;
//         out.y = self.y + (next.y - self.y) * s;
//         out.z = self.z + (next.z - self.z) * s;
//     }
// }
//
// impl AnimatableValue for Quat {
//     fn tl_lerp(&self, s: f32, next: &Self, out: &mut Self) {
//         *out = self.slerp(*next, s);
//     }
// }

pub struct Trans( Vec3 );

impl AnimatableValue<Transform> for Trans {
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        let mut out = &mut out.translation;
        out.x = self.0.x + (next.0.x - self.0.x) * s;
        out.y = self.0.y + (next.0.y - self.0.y) * s;
        out.z = self.0.z + (next.0.z - self.0.z) * s;
    }
}

pub struct Scale( Vec3 );

impl AnimatableValue<Transform> for Scale {
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        let mut out = &mut out.scale;
        out.x = self.0.x + (next.0.x - self.0.x) * s;
        out.y = self.0.y + (next.0.y - self.0.y) * s;
        out.z = self.0.z + (next.0.z - self.0.z) * s;
    }
}

pub struct Rotation( Quat );
impl AnimatableValue<Transform> for Rotation {
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        let mut out = &mut out.rotation;
        *out = self.0.slerp(next.0, s);
    }
}

pub struct TLTransform {
    pub scale: Option<Scale>,
    pub rotation: Option<Rotation>,
    pub translation: Option<Trans>,
}

impl AnimatableValue<Transform> for TLTransform {
    fn interpolate(&self, s: f32, next: &Self, out: &mut Transform) {
        if let (Some(scale), Some(next_scale)) = (self.scale.as_ref(), next.scale.as_ref()) {
            scale.interpolate(s, next_scale, out);
        }
        if let (Some(rot), Some(next_rot)) = (self.rotation.as_ref(), next.rotation.as_ref()) {
            rot.interpolate(s, next_rot, out);
        }
        if let (Some(translation), Some(next_translation)) = (self.translation.as_ref(), next.translation.as_ref()) {
            translation.interpolate(s, next_translation, out);
        }
    }
}