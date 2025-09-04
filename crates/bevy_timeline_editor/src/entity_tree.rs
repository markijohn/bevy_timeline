use std::collections::HashMap;
use bevy::{
    color::palettes::basic::RED,
    pbr::{ExtendedMaterial, MaterialExtensionKey, MaterialExtension, OpaqueRendererMethod},
    prelude::*,
    render::render_resource::*,
};
use bevy::app::{App, Plugin};
use bevy::color::Color;
use bevy::ecs::system::{SystemId, SystemParamItem};
use bevy::input::keyboard::KeyboardInput;
use bevy::math::Vec3;
use bevy::text::{JustifyText, TextColor, TextFont, TextLayout};
use bevy_mod_billboard::prelude::*;
use crate::shader::top::*;

#[derive(Clone, Copy)]
pub struct LockAxis {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl Default for LockAxis {
    fn default() -> Self {
        Self {
            x: false,
            y: false,
            z: false,
        }
    }
}

impl LockAxis {
    pub fn all() -> Self {
        Self { x: true, y: true, z: true }
    }

    pub fn none() -> Self {
        Self { x: false, y: false, z: false }
    }

    pub fn only_x() -> Self {
        Self { x: true, y: false, z: false }
    }

    pub fn only_y() -> Self {
        Self { x: false, y: true, z: false }
    }

    pub fn only_z() -> Self {
        Self { x: false, y: false, z: true }
    }

    pub fn except_x() -> Self {
        Self { x: false, y: true, z: true }
    }

    pub fn except_y() -> Self {
        Self { x: true, y: false, z: true }
    }

    pub fn except_z() -> Self {
        Self { x: true, y: true, z: false }
    }
}

#[derive(Resource, Default)]
pub enum MarkControlStatus {
    #[default]
    None,
    Translation { base:Vec2, lock_mode:LockAxis, origin:Vec<(Entity,Vec3)> },
    Rotation { base:Vec2, lock_mode:LockAxis, origin:Vec<(Entity,Quat)> },
}

pub struct TreeNode {
    pub name : String,
    pub entity : Entity,
    pub children : Vec<TreeNode>
}

#[derive(Resource)]
pub struct TargetRefreshFn(pub SystemId);

#[derive(Resource)]
pub struct TargetList {
    pub transform : Vec<TreeNode>,
    pub point_light : Vec<TreeNode>,
    pub spot_light : Vec<TreeNode>,
}

#[derive(Resource)]
struct MarkResource {
    mark_mesh : Handle<Mesh>,
    col_idle : Handle<AlwaysTopMaterial>,
    col_sel : Handle<AlwaysTopMaterial>
}

#[derive(Component)]
struct SelectedMark;

#[derive(Component)]
struct MarkTextEntity(Entity);

pub struct TimelineTargetListPlugin;


impl Plugin for TimelineTargetListPlugin {
    fn build(&self, app: &mut App) {
        let target_refresh = app.world_mut().register_system( make_treenode );
        app.insert_resource( TargetRefreshFn(target_refresh) );
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, AlwaysOnTopExt>,
        >::default());
        app.add_systems( Startup, setup);
        app.add_systems( Update, billboard_fit_scale );
        app.add_systems( Update, handle_mouse_click_with_radius.run_if(not(egui_wants_any_pointer_input)) );
        app.insert_resource( TargetList { transform:Vec::new(), point_light:Vec::new() , spot_light : Vec::new() } );

    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials: ResMut<Assets<AlwaysTopMaterial>>,
) {
    let mark_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.2));
    //let col_idle = materials.add(Color::srgb_u8(0, 200, 0));
    // let col_idle= materials.add(StandardMaterial {
    //     base_color : Color::srgb_u8(0, 200, 0),
    //     unlit:true,
    //     ..default()
    // });
    // let col_sel = materials.add(Color::srgb_u8(200, 0, 0));

    let col_idle = materials.add( AlwaysTopMaterial {
        base : StandardMaterial {
                base_color : Color::srgb_u8(0, 200, 0),
                unlit:true,
                ..default()
            },
        ..default()
    });

    let col_sel = materials.add( AlwaysTopMaterial {
        base : StandardMaterial {
            base_color : Color::srgb_u8(200, 0, 0),
            unlit:true,
            ..default()
        },
        ..default()
    });

    commands.insert_resource( MarkResource { mark_mesh, col_idle, col_sel } );

}

fn make_treenode(
    mut cmds : Commands,
    mark : Res<MarkResource>,
    mut target_list : ResMut<TargetList>,
    mut query_transform : Query<(&Name, Entity, Option<&Children>), With<Transform>>,
    mut query_point_light : Query<(&Name, Entity), With<PointLight>>,
    mut query_spot_light : Query<(&Name, Entity), With<SpotLight>>) {


    // PointLight와 SpotLight는 간단하게 처리
    target_list.point_light.clear();
    for (name, entity) in query_point_light.iter() {
        target_list.point_light.push(TreeNode {
            name: name.to_string(),
            entity,
            children: Vec::new(),
        });
    }

    target_list.spot_light.clear();
    for (name, entity) in query_spot_light.iter() {
        target_list.spot_light.push(TreeNode {
            name: name.to_string(),
            entity,
            children: Vec::new(),
        });
    }

    // Transform은 트리 구조로 처리
    target_list.transform.clear();


    // 모든 엔티티를 HashMap에 저장 (빠른 검색을 위해)
    let mut entity_map: HashMap<Entity, (&Name, Option<&Children>)> = HashMap::new();
    let mut all_child_entities = std::collections::HashSet::new();

    for (name, entity, children) in query_transform.iter() {
        entity_map.insert(entity, (name, children));

        // 모든 자식 엔티티들을 수집
        if let Some(children) = children {
            for child in children.iter() {
                all_child_entities.insert(child);
            }
        }
    }

    // 자식으로 참조되지 않는 엔티티들을 루트로 간주
    let mut root_entities = Vec::new();
    for (entity, _) in entity_map.iter() {
        if !all_child_entities.contains(entity) {
            root_entities.push(*entity);
        }
    }

    // 재귀적으로 트리를 구성하는 함수
    fn build_tree_recursive(
        cmds:&mut Commands,
        mark : &Res<MarkResource>,
        entity: Entity,
        entity_map: &HashMap<Entity, (&Name, Option<&Children>)>
    ) -> Option<TreeNode> {
        if let Some((name, children_opt)) = entity_map.get(&entity) {
            let mut entity_cmd = cmds.entity(entity);
            entity_cmd.with_children( |ecmds| {
                let text_entity = ecmds.spawn( (
                    BillboardText::new(name.as_str()),
                    // TextFont::default().with_font_size(60.0),
                    BillboardDepth(false),
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.0085)),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Visibility::Hidden
                ) ).id();


                ecmds.spawn( (
                    Mesh3d(mark.mark_mesh.clone()),
                    MeshMaterial3d(mark.col_idle.clone()),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    MarkTextEntity(text_entity),
                    Visibility::Visible
                ) )
                .observe( |selected:Trigger<OnAdd,SelectedMark>, mark : Res<MarkResource>, mut query:Query<&mut MeshMaterial3d<AlwaysTopMaterial>>| {
                    if let Ok(mut e) = query.get_mut( selected.target() ) {
                        e.0 = mark.col_sel.clone();
                    }
                })
                .observe( |selected:Trigger<OnRemove,SelectedMark>, mark : Res<MarkResource>, mut query:Query<&mut MeshMaterial3d<AlwaysTopMaterial>>| {
                    if let Ok(mut e) = query.get_mut( selected.target() ) {
                        e.0 = mark.col_idle.clone();
                    }
                });
                // .observe( |over:Trigger<Pointer<Over>>, mut query:Query<&MarkTextEntity>, mut text_query:Query<&mut Visibility, With<BillboardText>>| {
                //     let text_entity = query.get( over.target ).unwrap().0;
                //     let mut visibility = text_query.get_mut( text_entity).unwrap();
                //     *visibility = Visibility::Visible;
                // } ).observe( |out:Trigger<Pointer<Out>>, mut query:Query<&MarkTextEntity>, mut text_query:Query<&mut Visibility, With<BillboardText>>| {
                //     let text_entity = query.get( out.target ).unwrap().0;
                //     let mut visibility = text_query.get_mut( text_entity).unwrap();
                //     *visibility = Visibility::Hidden;
                // });
            });
            entity_cmd.with_child( (
                BillboardText::new(name.as_str()),
                // TextFont::default().with_font_size(60.0),
                BillboardDepth(false),
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.0085)),
                TextLayout::new_with_justify(JustifyText::Center),
                Visibility::Hidden
            ));


            let mut tree_node = TreeNode {
                name: name.to_string(),
                entity,
                children: Vec::new(),
            };

            // 자식 엔티티들을 재귀적으로 처리
            if let Some(children) = children_opt {
                for child_entity in children.iter() {
                    if let Some(child_node) = build_tree_recursive(cmds, mark, child_entity, entity_map) {
                        tree_node.children.push(child_node);
                    }
                }
            }

            Some(tree_node)
        } else {
            None
        }
    }

    // 루트 엔티티들부터 시작해서 트리 구성
    for root_entity in root_entities {
        if let Some(root_node) = build_tree_recursive(&mut cmds, &mark, root_entity, &entity_map) {
            target_list.transform.push(root_node);
        }
    }
}

// 또는 더 간단한 버전 (고정 base_scale 사용)
fn billboard_fit_scale(
    // mut query:ParamSet<(
    //     Query<(&mut Transform, &MarkTextEntity), Without<BillboardText> >,
    //     Query<(&mut Transform, &BillboardText), Without<MarkTextEntity> >
    // )>, //그냥 아래와 같이 논리적으로 Transform 에 대한 duplicate 쿼리가 나지 않을 다중 조건을 넣어주면 됨
    mut query_box: Query<(&mut Transform, &MarkTextEntity), Without<BillboardText>>,
    mut query_text: Query<(&mut Transform, &BillboardText), Without<MarkTextEntity>>,
    camera_query: Query<&Transform, (Changed<Transform>, With<Camera3d>, Without<MarkTextEntity>, Without<BillboardText>)>,

) {
    const BASE_TEXT_SCALE:f32 = 0.00085;
    const BASE_BOX_SCALE:f32 = 0.05;

    if let Ok( camera_transform ) = camera_query.single() {
        for mut transform in query_text.iter_mut() {
            let distance = camera_transform.translation.distance(transform.0.translation);
            let scale_factor = distance * BASE_TEXT_SCALE;
            transform.0.scale = Vec3::splat(scale_factor);
        }

        for mut transform in query_box.iter_mut() {
            let distance = camera_transform.translation.distance(transform.0.translation);
            let scale_factor = distance * BASE_BOX_SCALE;
            transform.0.scale = Vec3::splat(scale_factor);
        }
    }
}

use bevy::prelude::{Res,ButtonInput,MouseButton,Window,Camera,GlobalTransform,Vec2, Result};
use bevy::render::camera::ViewportConversionError;
use bevy::window::PrimaryWindow;
use bevy_egui::input::egui_wants_any_pointer_input;
use transform_gizmo_bevy::GizmoCamera;

fn handle_mouse_click_with_radius(
    mut commands:Commands,
    mut last_focus: Local<Option<Entity>>,
    key: Res<ButtonInput<KeyCode>>,
    button: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GizmoCamera>>,
    q_transforms: Query<(Entity, Option<&SelectedMark>, &MarkTextEntity, &GlobalTransform), With<Transform>>,
    mut billboard_text: Query<&mut Visibility, With<BillboardText>>,
) -> Result {
    const MAX_CLICK_DISTANCE: f32 = 35.0; // 최대 클릭 거리

    if let Some(cursor_pos) = q_windows.single()?.cursor_position() {

        let (camera, camera_transform) = q_camera.single()?;

        let closest = q_transforms
            .iter()
            .filter_map(|(entity,is_selected, mark_entity, transform)| {
                // 3D -> 2D 투영
                camera.world_to_viewport(camera_transform, transform.translation())
                    .and_then(|screen_pos| {
                        let distance = cursor_pos.distance(screen_pos);
                        if distance <= MAX_CLICK_DISTANCE {
                            Ok((entity, is_selected,mark_entity, distance))
                        } else {
                            Err(ViewportConversionError::PastFarPlane)
                        }
                    }).ok()
            })
            .min_by(|(_,_,_, a), (_,_,_, b)| a.partial_cmp(b).unwrap());

        let mut hide_text = None;
        match closest {
            Some((entity,is_selected, mark_entity, distance)) => {
                // println!("근접 마커: {:?}, 거리: {:.2}", mark_entity.0, distance);

                if button.just_pressed(MouseButton::Left) {
                    if key.pressed(KeyCode::ShiftLeft) {
                        if is_selected.is_some() {
                            commands.entity(entity).remove::<SelectedMark>();
                        } else {
                            commands.entity(entity).insert(SelectedMark);
                        }
                    } else {
                        for (next, _,_,_) in q_transforms.iter() {
                            commands.entity(next).remove::<SelectedMark>();
                        }
                        commands.entity(entity).insert_if_new(SelectedMark);
                    }
                }

                if let Some(entity) = *last_focus {
                    if entity == mark_entity.0 {
                        return Ok(())
                    }
                }
                let mut visible = billboard_text.get_mut( mark_entity.0 )?;
                *visible = Visibility::Visible;
                hide_text = *last_focus;
                *last_focus = Some(mark_entity.0);
            }
            None => {
                // println!("근접 마커 없음");
                if button.just_pressed(MouseButton::Left) {
                    for (next, next_selected,_,_) in q_transforms.iter() {
                        commands.entity(next).remove::<SelectedMark>();
                    }
                }
                hide_text = *last_focus;
                *last_focus = None;
            }
        }

        if let Some(bef_focus) = hide_text {
            let mut visible = billboard_text.get_mut( bef_focus )?;
            *visible = Visibility::Hidden;
        }
    }
    Ok(())
}








// Transform 제어 시스템
pub fn transform_control_system(
    mut commands: Commands,
    mut control_status: ResMut<MarkControlStatus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut selected_query: Query<&mut Transform, With<SelectedMark>>,
) -> Result {
    let window = windows.single()?;
    let cursor_pos = window.cursor_position().unwrap_or(Vec2::ZERO);

    // 현재 선택된 오브젝트의 Transform 가져오기
    let current_transform = selected_query.iter().next().copied();



    match control_status.as_mut() {

        MarkControlStatus::None => {
            // G키로 Translation 모드 진입
            if keyboard_input.just_pressed(KeyCode::KeyG) {
                if let Some(transform) = current_transform {
                    *control_status = MarkControlStatus::Translation {
                        base: cursor_pos,
                        origin: transform.translation,
                        lock_mode: LockAxis::all(),
                    };
                }
            }
            // R키로 Rotation 모드 진입
            else if keyboard_input.just_pressed(KeyCode::KeyR) {
                if let Some(transform) = current_transform {
                    *control_status = MarkControlStatus::Rotation {
                        base: cursor_pos,
                        origin: transform.rotation,
                        lock_mode: LockAxis::all(),
                        original_transform: transform,
                        trackball_mode: false,
                    };
                }
            }
        },

        MarkControlStatus::Translation { base, origin, lock_mode, original_transform } => {
            // 축 잠금 처리
            handle_axis_locking(&keyboard_input, lock_mode);

            // Translation 적용
            if let Ok(mut transform) = selected_query.get_single_mut() {
                let mouse_delta = cursor_pos - *base;
                let movement_factor = 0.01; // 이동 속도 조절

                let mut movement = Vec3::ZERO;
                if lock_mode.x { movement.x = mouse_delta.x * movement_factor; }
                if lock_mode.y { movement.y = -mouse_delta.y * movement_factor; } // Y축 반전
                if lock_mode.z { movement.z = mouse_delta.x * movement_factor; } // Z축은 X 마우스 움직임으로

                transform.translation = *origin + movement;
            }

            // 취소 처리
            if keyboard_input.just_pressed(KeyCode::Escape) ||
                mouse_button_events.read().any(|event| event.button == MouseButton::Right && event.state.is_pressed()) {
                // 원래 위치로 복원
                if let Ok(mut transform) = selected_query.get_single_mut() {
                    *transform = *original_transform;
                }
                *control_status = MarkControlStatus::Selected;
            }
            // 확정 처리 (마우스 왼쪽 클릭 또는 엔터)
            else if mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Enter) {
                *control_status = MarkControlStatus::Selected;
            }
        },

        MarkControlStatus::Rotation { base, origin, lock_mode, original_transform, trackball_mode } => {
            // R키를 다시 누르면 트랙볼 모드 토글
            if keyboard_input.just_pressed(KeyCode::KeyR) {
                *trackball_mode = !*trackball_mode;
            }

            // 축 잠금 처리
            handle_axis_locking(&keyboard_input, lock_mode);

            // Rotation 적용
            if let Ok(mut transform) = selected_query.get_single_mut() {
                let mouse_delta = cursor_pos - *base;
                let rotation_factor = 0.01; // 회전 속도 조절

                if *trackball_mode {
                    // 트랙볼 모드: 자유 회전
                    let rotation_x = Quat::from_axis_angle(Vec3::X, -mouse_delta.y * rotation_factor);
                    let rotation_y = Quat::from_axis_angle(Vec3::Y, -mouse_delta.x * rotation_factor);
                    transform.rotation = *origin * rotation_y * rotation_x;
                } else {
                    // 축 제한 모드
                    let mut rotation = Quat::IDENTITY;

                    if lock_mode.x {
                        rotation *= Quat::from_axis_angle(Vec3::X, -mouse_delta.y * rotation_factor);
                    }
                    if lock_mode.y {
                        rotation *= Quat::from_axis_angle(Vec3::Y, -mouse_delta.x * rotation_factor);
                    }
                    if lock_mode.z {
                        rotation *= Quat::from_axis_angle(Vec3::Z, mouse_delta.x * rotation_factor);
                    }

                    transform.rotation = *origin * rotation;
                }
            }

            // 취소 처리
            if keyboard_input.just_pressed(KeyCode::Escape) ||
                mouse_button_events.read().any(|event| event.button == MouseButton::Right && event.state.is_pressed()) {
                // 원래 상태로 복원
                if let Ok(mut transform) = selected_query.get_single_mut() {
                    *transform = *original_transform;
                }
                *control_status = MarkControlStatus::Selected;
            }
            // 확정 처리
            else if mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Enter) {
                *control_status = MarkControlStatus::Selected;
            }
        },
    }
}
//
// fn handle_axis_locking(keyboard_input: &Res<ButtonInput<KeyCode>>, lock_mode: &mut LockAxis) {
//     let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
//
//     if keyboard_input.just_pressed(KeyCode::KeyX) {
//         if shift_pressed {
//             // Shift + X: X축 제외하고 잠금
//             *lock_mode = LockAxis::except_x();
//         } else {
//             // X: X축만 잠금
//             *lock_mode = LockAxis::only_x();
//         }
//     }
//     else if keyboard_input.just_pressed(KeyCode::KeyY) {
//         if shift_pressed {
//             // Shift + Y: Y축 제외하고 잠금
//             *lock_mode = LockAxis::except_y();
//         } else {
//             // Y: Y축만 잠금
//             *lock_mode = LockAxis::only_y();
//         }
//     }
//     else if keyboard_input.just_pressed(KeyCode::KeyZ) {
//         if shift_pressed {
//             // Shift + Z: Z축 제외하고 잠금
//             *lock_mode = LockAxis::except_z();
//         } else {
//             // Z: Z축만 잠금
//             *lock_mode = LockAxis::only_z();
//         }
//     }
// }