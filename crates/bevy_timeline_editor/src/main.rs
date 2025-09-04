use std::f32::consts::PI;
use bevy::asset::{RenderAssetUsages, UnapprovedPathMode};
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_egui::egui::{Frame, Id, Margin, Popup, PopupCloseBehavior, ScrollArea, Widget};
use bevy_timeline_runtime::prelude::*;
use bevy_timeline_impls::prelude::*;
use transform_gizmo_bevy::*;
use bevy_mod_outline::*;
use crate::editor::TimelineEditor;
use crate::entity_tree::{TargetList, TargetRefreshFn, TimelineTargetListPlugin, TreeNode};
use crate::sample_scene::{SampleScenePlugin, SpawnSampleScene};

mod camera;
mod picking;
mod editor;
mod sample_scene;
mod entity_tree;
mod shader;

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
        .add_plugins(bevy_mod_billboard::prelude::BillboardPlugin)
        .add_plugins(camera::PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(TransformGizmoPlugin)
        .add_plugins(picking::GizmoPickingPlugin)
        .insert_resource(GizmoOptions {
            hotkeys: Some(GizmoHotkeys::default()),
            ..default()
        })
        .add_plugins(TimelinePlugin::<TimelineSet>::new())
        .add_plugins(SampleScenePlugin)
        .add_plugins(TimelineTargetListPlugin)
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, draw_egui)
    .run();
}

#[derive(Resource,Default)]
pub struct AnimationData(Vec<Handle<TimelineAnimationSet>>);


fn setup(
    mut gizmo_options: ResMut<GizmoOptions>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    commands.spawn((
        Name::new("DefaultPointLight"),
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
        Visibility::default()
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
    mut cmds: Commands,
    target_refresh_fn: Res<TargetRefreshFn>,
    target_list: Res<TargetList>,
    sample_scene_spawner: Res<SpawnSampleScene>,
    mut anim_datas:ResMut<AnimationData>,
    mut timeline_editor: ResMut<TimelineEditor>,
    mut scene_assets: ResMut<Assets<Scene>>,
    mut anim_sets:ResMut<Assets<TimelineAnimationSet>>,
    mut anim_assets:ResMut<Assets<TimelineAnimation>>,
    mut asset_server: ResMut<AssetServer>,
    mut egui_context: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>
) -> Result {
    let ctx_mut = egui_context.ctx_mut()?;

    let mut top = egui::TopBottomPanel::top("timeline_top_panel")
        .show(ctx_mut, |ui| {
            ui.horizontal(|ui| {
                let response = ui.button("File");
                Popup::menu(&response)
                    .id( Id::new("file_popup") )
                    .gap(4.)
                    .close_behavior(PopupCloseBehavior::CloseOnClick)
                    .show( |ui| {
                        ui.set_max_width(200.);
                        if ui.button("Open file").clicked() {
                            if let Some(files) = rfd::FileDialog::new().pick_files() {
                                for file in files {
                                    let handle:Handle<Scene> = asset_server.load( format!( "{}#Scene0", file.to_str().unwrap() ) );

                                    cmds.spawn( (
                                        SceneRoot(handle),
                                        Visibility::default()
                                    ) );
                                }
                            }
                        }
                        ui.menu_button("Load sample", |ui| {
                            if ui.button("Shapes").clicked() {
                                cmds.run_system( sample_scene_spawner.shapes_fn_id );
                            }
                            if ui.button("Human").clicked() {
                                cmds.run_system( sample_scene_spawner.human_fn_id );
                            }
                        })
                    });
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
        .frame( Frame::new()
            .inner_margin(Margin::symmetric(0, 0))
            .fill(ctx_mut.theme().default_visuals().panel_fill) )
        .resizable(true)
        .show(ctx_mut, |ui| {
            ui.set_height(ui.available_height());
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

    // let mut left = egui::SidePanel::left("timeline_entities")
    //     .min_width(200.)
    //     .max_width(1000.)
    //     .resizable(true)
    //     .show(ctx_mut, |ui| {
    //
    //     }).response.rect.width();
    let mut left = 0.;

    let mut right = egui::SidePanel::right("timeline_attr")
        .show(ctx_mut, |ui| {
            ui.set_width(ui.available_width());
            if ui.button("Refresh").clicked() {
                cmds.run_system( target_refresh_fn.0 );
            }
            ui.separator();

            fn recurrsive_tree(ui:&mut egui::Ui, tree_node:&TreeNode) {
                if tree_node.children.len() == 0 {
                    ui.label(tree_node.name.as_str());
                } else {
                    ui.collapsing(tree_node.name.as_str(), |ui| {
                        for child in tree_node.children.iter() {
                            recurrsive_tree(ui, child);
                        }
                    });
                }
            }
            ScrollArea::vertical()
                .show(ui, |ui| {
                    ui.collapsing(format!("Transform ({})", target_list.transform.len() ), |ui| {
                        for child in target_list.transform.iter() {
                            recurrsive_tree(ui, child);
                        }
                    }).openness = 1.0;
                    ui.collapsing(format!("PointLight ({})",target_list.point_light.len()), |ui| {
                        for light in target_list.point_light.iter() {
                            ui.label( &light.name );
                        }
                    }).openness = 1.0;
                });
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