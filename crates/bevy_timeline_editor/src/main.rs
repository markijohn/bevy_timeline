use std::f32::consts::PI;
use std::str::FromStr;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_timeline_runtime::prelude::*;
use bevy_timeline_impls::prelude::*;

type TimelineSet = (TransformSet,StdMaterialSet,DirLightSet,PointLightSet,SpotLightSet,);
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins( TimelinePlugin::<TimelineSet>::new() )
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, draw_egui)
    .run();
}

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

fn setup(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

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
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,
                Z_EXTENT / 2.,
            )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    let num_extrusions = extrusions.len();

    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
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
        ));
    }

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(bevy::color::palettes::basic::SILVER))),
    ));

    //World camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    // Egui camera.
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));
}

fn draw_egui(
    time: Res<Time>,
    mut egui_context: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>
) -> Result {
    let ctx_mut = egui_context.ctx_mut()?;

    let mut top = egui::TopBottomPanel::top("timeline_top_panel")
        .min_height(30.)
        .max_height(30.0)
        .show(ctx_mut, |ui| {
            ui.label("timeline_top_panel");
        }).response.rect.height();

    let mut bottom = egui::TopBottomPanel::bottom("timeline_bottom_panel")
        .min_height(100.)
        .max_height(1000.0)
        .resizable(true)
        .show(ctx_mut, |ui| {
            if ui.button("Load").clicked() {
                if let Some(files) = rfd::FileDialog::new().pick_files() {
                    for file in files {
                        let string = std::fs::read_to_string(file).unwrap();
                        if let Ok(value) = serde_json::Value::from_str(&string) {
                            if let Ok(anims) = TimelineAnimation::load_animations::<TimelineSet>( &value ) {

                            } else {
                                error!("TimelineAnimation parse failed");
                            }
                        } else {
                            error!("json load error");
                        }
                    }
                }
            }
        }).response.rect.height();

    let mut left = egui::SidePanel::left("timeline_entities")
        .min_width(200.)
        .max_width(1000.)
        .show(ctx_mut, |ui| {

        }).response.rect.width();

    let mut right = egui::SidePanel::right("timeline_attr")
        .show(ctx_mut, |ui| {

        }).response.rect.width();

    // Scale from logical units to physical units.
    left *= window.scale_factor();
    right *= window.scale_factor();
    top *= window.scale_factor();
    bottom *= window.scale_factor();

    let pos = UVec2::new(left as u32, top as u32);
    let size = UVec2::new(window.physical_width(), window.physical_height())
        - pos
        - UVec2::new(right as u32, bottom as u32);

    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });

    Ok(())
}