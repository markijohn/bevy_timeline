use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

pub trait AnimatableValue<Out=Self> {
    fn tl_lerp(&self, s:f32, next:&Self, out:&mut Out);
}

impl AnimatableValue for f32 {
    fn tl_lerp(&self, s:f32, next:&f32, out:&mut f32) {
        *out = self + (next - self) * s;
    }
}

impl AnimatableValue for Vec3 {
    fn tl_lerp(&self, s: f32, next: &Self, out: &mut Self) {
        out.x = self.x + (next.x - self.x) * s;
        out.y = self.y + (next.y - self.y) * s;
        out.z = self.z + (next.z - self.z) * s;
    }
}

impl AnimatableValue for Quat {
    fn tl_lerp(&self, s: f32, next: &Self, out: &mut Self) {
        *out = self.slerp(*next, s);
    }
}

pub struct Transition( Vec3 );

impl AnimatableValue<Vec3> for Transition {
    fn tl_lerp(&self, s: f32, next: &Self, out: &mut Vec3) {
        out.x = self.0.x + (next.0.x - self.0.x) * s;
        out.y = self.0.y + (next.0.y - self.0.y) * s;
        out.z = self.0.z + (next.0.z - self.0.z) * s;
    }
}

pub struct Scale( Vec3 );

impl AnimatableValue<Vec3> for Scale {
    fn tl_lerp(&self, s: f32, next: &Self, out: &mut Vec3) {
        out.x = self.0.x + (next.0.x - self.0.x) * s;
        out.y = self.0.y + (next.0.y - self.0.y) * s;
        out.z = self.0.z + (next.0.z - self.0.z) * s;
    }
}

pub struct Rotation( Quat );
impl AnimatableValue<Quat> for Rotation {
    fn tl_lerp(&self, s: f32, next: &Self, out: &mut Quat) {
        self.0.tl_lerp(s, &next.0, out);
    }
}

pub struct Trnas {
    pub scale: Option<Scale>,
    pub rotation: Option<Rotation>,
    pub translation: Option<Vec3>,
}

impl AnimatableValue<Transform> for Trnas {
    fn tl_lerp(&self, s: f32, next: &Self, out: &mut Transform) {
        if let (Some(scale), Some(next_scale)) = (self.scale.as_ref(), next.scale.as_ref()) {
            scale.tl_lerp(s, next_scale, &mut out.scale);
        }
        if let (Some(rot), Some(next_rot)) = (self.rotation.as_ref(), next.rotation.as_ref()) {
            rot.tl_lerp(s, next_rot, &mut out.rotation);
        }
        if let (Some(translation), Some(next_translation)) = (self.translation.as_ref(), next.translation.as_ref()) {
            translation.tl_lerp(s, next_translation, &mut out.translation);
        }
    }
}