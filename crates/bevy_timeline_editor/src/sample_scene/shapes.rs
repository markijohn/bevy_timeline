use std::f32::consts::PI;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::Color;
use bevy::image::Image;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{MeshMaterial3d, PointLight, StandardMaterial};
use bevy::prelude::{default, Annulus, Camera, Camera2d, Camera3d, Capsule2d, Capsule3d, Circle, Commands, Component, Cone, ConicalFrustum, Cuboid, Cylinder, Ellipse, EventReader, Extrusion, Mesh, Mesh3d, Meshable, Name, Plane3d, Rectangle, RegularPolygon, ResMut, Sphere, Tetrahedron, Torus, Transform, Triangle2d, World};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::RenderLayers;
use bevy_egui::{EguiGlobalSettings, PrimaryEguiContext};
use bevy_mod_outline::{ComputedOutline, OutlineMode, OutlineStencil, OutlineVolume};
// use transform_gizmo_bevy::{GizmoCamera, GizmoMode, GizmoOptions};
// use crate::picking;

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}


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
        (Name::new("Cuboid"), meshes.add(Cuboid::default()) ),
        (Name::new("Tetrahedron"), meshes.add(Tetrahedron::default()) ),
        (Name::new("Capsule3d"), meshes.add(Capsule3d::default()) ),
        (Name::new("Torus"), meshes.add(Torus::default()) ),
        (Name::new("Cylinder"), meshes.add(Cylinder::default()) ),
        (Name::new("Cone"), meshes.add(Cone::default()) ),
        (Name::new("ConicalFrustum"), meshes.add(ConicalFrustum::default()) ),
        (Name::new("Sphere_ico"), meshes.add(Sphere::default().mesh().ico(5).unwrap()) ),
        (Name::new("Sphere_uv"), meshes.add(Sphere::default().mesh().uv(32, 18)) ),
    ];

    let extrusions = [
        (Name::new("Rectangle"), meshes.add(Extrusion::new(Rectangle::default(), 1.)) ),
        (Name::new("Capsule2d"), meshes.add(Extrusion::new(Capsule2d::default(), 1.)) ),
        (Name::new("Annulus"), meshes.add(Extrusion::new(Annulus::default(), 1.)) ),
        (Name::new("Circle"), meshes.add(Extrusion::new(Circle::default(), 1.)) ),
        (Name::new("Ellipse"), meshes.add(Extrusion::new(Ellipse::default(), 1.)) ),
        (Name::new("RegularPolygon"), meshes.add(Extrusion::new(RegularPolygon::default(), 1.)) ),
        (Name::new("Triangle2d"), meshes.add(Extrusion::new(Triangle2d::default(), 1.)) ),
    ];

    let num_shapes = shapes.len();

    for (i, (name,shape)) in shapes.into_iter().enumerate() {
        commands.spawn((
            name,
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,
                Z_EXTENT / 2.,
            )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let num_extrusions = extrusions.len();

    for (i, (name,shape)) in extrusions.into_iter().enumerate() {
        commands.spawn((
            name,
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                -Z_EXTENT / 2.,
            )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,

            // Pick,
            OutlineVolume {
                visible: false,
                colour: Color::WHITE,
                width: 2.0,
            },
            // picking::PickSelection { is_selected: true },
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