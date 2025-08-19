use std::f32::consts::PI;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::image::Image;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{MeshMaterial3d, PointLight, StandardMaterial};
use bevy::prelude::{default, Annulus, Camera, Camera2d, Camera3d, Capsule2d, Capsule3d, Circle, Commands, Component, Cone, ConicalFrustum, Cuboid, Cylinder, Ellipse, EventReader, Extrusion, Mesh, Mesh3d, Meshable, Plane3d, Rectangle, RegularPolygon, ResMut, Sphere, Tetrahedron, Torus, Transform, Triangle2d, World};
use bevy::render::view::RenderLayers;
use bevy_egui::{EguiGlobalSettings, PrimaryEguiContext};
use bevy_mod_outline::{ComputedOutline, OutlineMode, OutlineStencil, OutlineVolume};
use transform_gizmo_bevy::{GizmoCamera, GizmoMode, GizmoOptions};
use crate::{camera, picking, uv_debug_texture};
use crate::sample_scene::SpawnSampleScene;

#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

pub fn setup_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Tetrahedron::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cone::default()),
        meshes.add(ConicalFrustum::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];

    let extrusions = [
        meshes.add(Extrusion::new(Rectangle::default(), 1.)),
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
        meshes.add(Extrusion::new(Annulus::default(), 1.)),
        meshes.add(Extrusion::new(Circle::default(), 1.)),
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -crate::SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * crate::SHAPES_X_EXTENT,
                2.0,
                crate::Z_EXTENT / 2.,
            )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            crate::Shape,
        ));
    }

    let num_extrusions = extrusions.len();

    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -crate::EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * crate::EXTRUSION_X_EXTENT,
                2.0,
                -crate::Z_EXTENT / 2.,
            )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            crate::Shape,

            // Pick,
            OutlineVolume {
                visible: false,
                colour: Color::WHITE,
                width: 2.0,
            },
            picking::PickSelection { is_selected: true },
            OutlineStencil::default(),
            OutlineMode::default(),
            ComputedOutline::default(),
        ));
    }

    

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(bevy::color::palettes::basic::SILVER))),
    ));
}