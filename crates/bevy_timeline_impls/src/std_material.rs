use bevy_asset::Assets;
use bevy_ecs::prelude::{Changed, Component, Query, ResMut};
use bevy_color::{Color, Mix};
use bevy_math::FloatExt;
use serde::{Serialize, Deserialize};
use bevy_timeline_runtime::prelude::{AnimatableValue, TimelineError};

use bevy_pbr::{MeshMaterial3d, StandardMaterial};

pub type StdMaterialSet = (StdMaterialColor,);

pub fn std_material_control(
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&MeshMaterial3d<StandardMaterial>, &mut MaterialController), Changed<MaterialController>>,
) {
    for (mesh_material, mut controller) in query.iter_mut() {
        if let Some(mat) = assets.get_mut( &mesh_material.0 ) {
            if let Some(color) = controller.base_color.take() {
                mat.base_color = color;
            }
            if let Some(v) = controller.perceptual_roughness.take() {
                mat.perceptual_roughness = v;
            }
            if let Some(v) = controller.metallic.take() {
                mat.metallic = v;
            }
            if let Some(v) = controller.reflectance.take() {
                mat.reflectance = v;
            }
            if let Some(v) = controller.thickness.take() {
                mat.thickness = v;
            }
            if let Some(v) = controller.ior.take() {
                mat.ior = v;
            }
            if let Some(v) = controller.attenuation_color.take() {
                mat.attenuation_color = v;
            }
            if let Some(v) = controller.clearcoat.take() {
                mat.clearcoat = v;
            }
            if let Some(v) = controller.clearcoat_perceptual_roughness.take() {
                mat.clearcoat_perceptual_roughness = v;
            }
        }
    }
}

#[derive(Component)]
pub struct MaterialController {
    base_color: Option<Color>,
    perceptual_roughness: Option<f32>,
    metallic: Option<f32>,
    reflectance: Option<f32>,
    thickness: Option<f32>,
    ior: Option<f32>,
    attenuation_color: Option<Color>,
    clearcoat: Option<f32>,
    clearcoat_perceptual_roughness: Option<f32>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct StdMaterialColor(pub Color);

impl AnimatableValue for StdMaterialColor {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.base_color = Some(start.0.mix( &end.0, s ));
        } else {
            out.base_color = Some(start.0);
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct StdMaterialRoughness(pub f32);

impl AnimatableValue for StdMaterialRoughness {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.perceptual_roughness = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct StdMaterialMetallic(pub f32);

impl AnimatableValue for StdMaterialMetallic {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.metallic = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StdMaterialReflectance(pub f32);

impl AnimatableValue for StdMaterialReflectance {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.reflectance = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StdMaterialThickness(pub f32);

impl AnimatableValue for StdMaterialThickness {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.thickness = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StdMaterialIor(pub f32);

impl AnimatableValue for StdMaterialIor {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.ior = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StdMaterialAttColor(pub Color);

impl AnimatableValue for StdMaterialAttColor {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.attenuation_color = Some(start.0.mix( &end.0, s ));
        } else {
            out.attenuation_color = Some(start.0);
        }
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StdMaterialClearCoat(pub f32);

impl AnimatableValue for StdMaterialClearCoat {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.clearcoat = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct StdMaterialClearCoatRough(pub f32);

impl AnimatableValue for StdMaterialClearCoatRough {
    type Target = MaterialController;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        out.clearcoat_perceptual_roughness = Some( start.0.lerp( end.unwrap_or_default().0, s ) );
    }
}