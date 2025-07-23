use bevy_asset::Assets;
use bevy_ecs::prelude::{Changed, Component, Query, ResMut};
use bevy_color::{Color, Mix};
use serde_json::{Value};
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

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
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

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        Ok( Self(
            serde_json::from_value( value.clone() )?
        ) )
    }

    fn to_value(&self) -> Value {
        serde_json::to_value( self.0 ).unwrap()
    }
}