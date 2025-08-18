use std::f32::consts::PI;
use bevy::asset::{RenderAssetUsages, UnapprovedPathMode};
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_egui::egui::Widget;
use bevy_timeline_runtime::prelude::*;
use bevy_timeline_impls::prelude::*;
use transform_gizmo_bevy::*;
use bevy_mod_outline::*;
use crate::editor::TimelineEditor;

mod camera;
mod picking;
mod editor;

//type TimelineSet = (TransformSet,StdMaterialSet,DirLightSet,PointLightSet,SpotLightSet,);
type TimelineSet = (DefaultTransformSet,);

fn main() {
    App::new()
        .init_resource::<AnimationData>()
        .init_resource::<TimelineEditor>()
        .add_plugins(DefaultPlugins
                         .set(AssetPlugin {
                             unapproved_path_mode: UnapprovedPathMode::Allow,
                             ..default()
                         })
        )
        .add_plugins(camera::PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(TransformGizmoPlugin)
        .add_plugins(picking::GizmoPickingPlugin)
        .insert_resource(GizmoOptions {
            hotkeys: Some(GizmoHotkeys::default()),
            ..default()
        })
        .add_plugins(TimelinePlugin::<TimelineSet>::new())
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, draw_egui)
    .run();
}

#[derive(Resource,Default)]
pub struct AnimationData(Vec<Handle<TimelineAnimationSet>>);

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
    mut gizmo_options: ResMut<GizmoOptions>,
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    gizmo_options.gizmo_modes.remove( GizmoMode::ScaleX );
    gizmo_options.gizmo_modes.remove( GizmoMode::ScaleY );
    gizmo_options.gizmo_modes.remove( GizmoMode::ScaleZ );
    gizmo_options.visuals.gizmo_size = 75. / 1.3;

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
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    //     GizmoCamera,
    // ));
    commands.spawn((
        camera::PanOrbitCamera {
            // radius: camera_transform.translation.length(),
            ..Default::default()
        },
        Camera3d::default(),
        // camera_transform.looking_at(Vec3::ZERO, Vec3::Y),
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        GizmoCamera,
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
    mut anim_datas:ResMut<AnimationData>,
    mut timeline_editor: ResMut<TimelineEditor>,
    mut anim_sets:ResMut<Assets<TimelineAnimationSet>>,
    mut anim_assets:ResMut<Assets<TimelineAnimation>>,
    mut asset_server: ResMut<AssetServer>,
    mut egui_context: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>
) -> Result {
    let ctx_mut = egui_context.ctx_mut()?;

    let mut top = egui::TopBottomPanel::top("timeline_top_panel")
        .min_height(30.)
        .max_height(30.0)
        .show(ctx_mut, |ui| {
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {

                }
                if ui.button("Load").clicked() {
                    if let Some(files) = rfd::FileDialog::new().pick_files() {
                        
                        for file in files {
                            let handle = asset_server.load( file );
                            anim_datas.0.push( handle );
                            
                        }
                    }
                }
                if ui.button("Mapper").clicked() {
                    
                }
            });

        }).response.rect.height();

    let mut bottom = egui::TopBottomPanel::bottom("timeline_bottom_panel")
        .min_height(200.)
        .max_height(1000.0)
        .resizable(true)
        .show(ctx_mut, |ui| {
            ui.horizontal(|ui| {
                let mut datas = Vec::new();
                for lazy_anim_set in anim_datas.0.iter() {
                    if let Some(anim) = anim_sets.get( lazy_anim_set ) {
                        datas.push( anim );
                    }
                }
                timeline_editor.ui(ui, datas, &mut anim_assets); 
            });
            egui::Separator::default().spacing(0.).ui( ui );
        }).response.rect.height();

    let mut left = egui::SidePanel::left("timeline_entities")
        .min_width(200.)
        .max_width(1000.)
        .resizable(true)
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