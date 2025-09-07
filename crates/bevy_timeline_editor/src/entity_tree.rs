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

#[derive(Clone)]
pub enum RotationMode {
    World,
    Local,
    LockWorld,
    LockLocal,
}

#[derive(Clone,Default)]
pub enum RotationAxis {
    #[default]
    None,
    X(RotationMode),
    Y(RotationMode),
    Z(RotationMode),
}

#[derive(Resource, Default)]
pub enum MarkControlStatus {
    #[default]
    None,
    Translation { base:Vec2, lock_mode:LockAxis, origin:Vec<(Entity,Vec3)> },
    Rotation { base:Vec2, axis_mode:RotationAxis, origin:Vec<(Entity,Quat,Vec3)>, trackball_mode:bool },
}

impl MarkControlStatus {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false
        }
    }
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
        app.insert_resource( MarkControlStatus::None );
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, AlwaysOnTopExt>,
        >::default());
        app.add_systems( Startup, setup);
        app.add_systems( Update, billboard_fit_scale );
        app.add_systems( Update, transform_control_system.after(handle_mouse_click_with_radius) );
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
    control_status: Res<MarkControlStatus>,
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
                if button.just_pressed(MouseButton::Left) && control_status.is_none() {
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





/// 화면 좌표를 가상의 구면(Arcball)에 투영
fn project_to_sphere(p: Vec2, size: Vec2) -> Vec3 {
    let mut v = Vec3::new(
        (2.0 * p.x - size.x) / size.x,
        (size.y - 2.0 * p.y) / size.y,
        0.0,
    );
    let d = v.x * v.x + v.y * v.y;
    if d <= 1.0 {
        v.z = (1.0 - d).sqrt();
    } else {
        v = v.normalize();
    }
    v
}

// Transform 제어 시스템
pub fn transform_control_system(
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut last_cursor_pos: Local<Vec2>,
    mut commands: Commands,
    mut control_status: ResMut<MarkControlStatus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut global_query: Query<&mut GlobalTransform, Without<Camera3d>>,
    mut query: Query<(Entity, &mut Transform)>,
    mut selected_query: Query<&ChildOf, With<SelectedMark>>,
) -> Result {
    let (camera, cam_transform) = cameras.single()?; // 주 카메라
    let window = windows.single()?;
    let cursor_pos = window.cursor_position().unwrap_or(Vec2::ZERO);
    let is_cursor_moved = (cursor_pos - *last_cursor_pos) != Vec2::ZERO;
    *last_cursor_pos = cursor_pos;

    match control_status.as_mut() {

        MarkControlStatus::None => {
            // G키로 Translation 모드 진입
            if keyboard_input.just_pressed(KeyCode::KeyG) {
                let origin:Vec<_> = selected_query.iter().map( |parent| {
                    let (parent_entity, transform) = query.get( parent.0 ).unwrap();
                    (parent_entity, transform.translation)
                }).collect();
                let len = origin.len();
                if len > 0 {
                    *control_status = MarkControlStatus::Translation {
                        base: cursor_pos,
                        lock_mode: LockAxis::all(),
                        origin,
                    };
                }
            }
            // R키로 Rotation 모드 진입
            else if keyboard_input.just_pressed(KeyCode::KeyR) {
                let origin:Vec<_> = selected_query.iter().map( |parent| {
                    let (parent_entity, transform) = query.get( parent.0 ).unwrap();
                    (parent_entity, transform.rotation.clone(), transform.translation)
                }).collect();
                let len = origin.len();
                if len > 0 {
                    *control_status = MarkControlStatus::Rotation {
                        base: cursor_pos,
                        origin,
                        axis_mode: RotationAxis::None,
                        trackball_mode: false,
                    };
                }
            }
        },

        MarkControlStatus::Translation { base, origin, lock_mode } => {
            // 축 잠금 처리
            handle_axis_locking(&keyboard_input, lock_mode);

            let diff = cursor_pos - *base;

            // Translation 적용
            if is_cursor_moved {
                let ray_from = camera.viewport_to_world(cam_transform, *base).unwrap();
                let ray_to = camera.viewport_to_world(cam_transform, cursor_pos).unwrap();

                // 평면: 카메라 뷰 평면 (camera forward와 수직)
                let plane_normal = cam_transform.forward();
                let plane_origin = origin[0].1; // 기준점 하나 선택 (첫 번째 선택된 엔티티 위치)

                // 두 점을 평면과 교차시켜 "월드 좌표" 얻기
                let start_hit = ray_from.intersect_plane(plane_origin, InfinitePlane3d { normal: plane_normal })
                    .map(|d| ray_from.origin + ray_from.direction * d);
                let now_hit = ray_to.intersect_plane(plane_origin, InfinitePlane3d { normal: plane_normal })
                    .map(|d| ray_to.origin + ray_to.direction * d);

                if let (Some(start), Some(now)) = (start_hit, now_hit) {
                    let delta = now - start;

                    for (entity, base_translation) in origin.iter() {
                        if let Ok( (e,mut tr) ) = query.get_mut(*entity) {
                            let mut t = *base_translation + delta;

                            // 축 잠금
                            if !lock_mode.x { t.x = base_translation.x; }
                            if !lock_mode.y { t.y = base_translation.y; }
                            if !lock_mode.z { t.z = base_translation.z; }

                            tr.translation = t;
                        }
                    }
                }
            }

            // 취소 처리
            if keyboard_input.just_pressed(KeyCode::Escape) || mouse_input.just_pressed(MouseButton::Right) {
                for (parent, origin) in origin.iter_mut() {
                    let (_parent, mut transform) = query.get_mut(*parent)?;
                    transform.translation = *origin;
                }
                *control_status = MarkControlStatus::None;
            }
            // 확정 처리 (마우스 왼쪽 클릭 또는 엔터)
            else if mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Enter) {
                *control_status = MarkControlStatus::None;
            }
        },

        MarkControlStatus::Rotation { base, origin, axis_mode, trackball_mode } => {
            // R키를 다시 누르면 트랙볼 모드 토글
            if keyboard_input.just_pressed(KeyCode::KeyR) {
                *trackball_mode = !*trackball_mode;
            }

            // 축 잠금 처리
            handle_rotation_axis(&keyboard_input, axis_mode);

            let diff = cursor_pos - *base;

            // Rotation 적용
            if is_cursor_moved {
                const rotation_factor:f32 = 0.01; // 이동 속도 조절

                if *trackball_mode {
                    // 트랙볼 모드: 자유 회전
                    // let rotation_x = Quat::from_axis_angle(Vec3::X, -diff.y * rotation_factor);
                    // let rotation_y = Quat::from_axis_angle(Vec3::Y, -diff.x * rotation_factor);
                    // target_transform.rotation = *base * rotation_y * rotation_x;

                    // Arcball 회전
                    let v0 = project_to_sphere(*base, window.size());
                    let v1 = project_to_sphere(cursor_pos, window.size());
                    let axis = v0.cross(v1).normalize_or_zero();
                    if axis.length_squared() > 0.0 {
                        let angle = v0.dot(v1).clamp(-1.0, 1.0).acos();
                        let q = Quat::from_axis_angle(axis, angle);

                        for (parent, base_rot, base_translation) in origin.iter() {
                            let (_, mut transform) = query.get_mut(*parent)?;
                            transform.rotation = *base_rot * q;
                        }
                    }
                } else {
                    // 축 제한 모드
                    // 1. Pivot (선택된 객체들의 월드 중심 → 2D 투영)
                    let mut world_center = Vec3::ZERO;
                    let mut count = 0;
                    //GlobalTransform 은 PostUpdate 에서 계산되므로 요구 이후에만 제대로 나옴
                    //때문에 아래의 world_to_viewport 는 Err 이 나올것
                    for (entity, _,_) in origin.iter() {
                        if let Ok(transform) = global_query.get(*entity) {
                            world_center += transform.translation();
                        }
                    }
                    world_center /= count as f32;

                    let origin: Vec<_> = selected_query.iter().filter_map(|child_of| {
                        let ent = child_of.0;
                        // global transform 가져오기 (월드 좌표)
                        if let Ok(g) = global_query.get(ent) {
                            // g.translation() / g.rotation() 메서드 이름은 bevy 버전에 따라 다를 수 있음.
                            let base_pos = g.translation();
                            let base_rot = g.rotation();
                            Some((child_of.0, base_rot, base_pos))
                        } else {
                            None
                        }
                    }).collect();

                    // 2D 투영 (Pivot 스크린 좌표)
                    // let Ok(pivot_2d) = camera.world_to_viewport(cam_transform, world_center) else { return Ok(()) };

                    // 2) 화면상 pivot 좌표 얻기 (카메라 API)
                    if let Ok(pivot_2d) = camera.world_to_viewport(cam_transform, world_center) {
                        let v1 = *base - pivot_2d;
                        let v2 = cursor_pos - pivot_2d;
                        if v1.length_squared() > 0.0 && v2.length_squared() > 0.0 {
                            // 안정적 부호 각도: atan2(cross, dot)
                            let cross = v1.x * v2.y - v1.y * v2.x;
                            let dot = v1.dot(v2);
                            let signed_angle = cross.atan2(dot);

                            // 회전축: 카메라 앞방향 (부호가 뒤집히면 axis에 -를 붙이세요)
                            let axis = -cam_transform.forward().normalize();
                            let rot = Quat::from_axis_angle(axis, signed_angle);

                            // 3) 각 엔티티에 대해: 월드 기준 새 pos/rot 계산 후 로컬로 변환해서 저장
                            for (entity, base_rot, base_pos) in origin.iter() {
                                // 월드에서의 새 회전/위치
                                let new_global_rot = *base_rot * rot;
                                let new_global_pos = world_center + rot * (*base_pos - world_center);

                                // 부모가 있는 경우: 부모의 GlobalTransform을 얻어 역변환 적용하여 로컬 변환 계산
                                if let Some(parent_ent) = parent_opt {
                                    if let Ok(parent_gt) = global_query.get(*parent_ent) {
                                        // parent_gt 의 월드 행렬 구해서 역행렬로 로컬 매트릭스 계산
                                        let parent_mat = parent_gt.compute_matrix();
                                        let inv = parent_mat.inverse();

                                        let new_global_mat =
                                            Mat4::from_scale_rotation_translation(Vec3::ONE, new_global_rot, new_global_pos);
                                        let new_local_mat = inv * new_global_mat;

                                        // Transform::from_matrix 가 없으면 직접 분해해서 translation/rotation/scale 할 것
                                        let new_local = Transform::from_matrix(new_local_mat);

                                        if let Ok((_, mut tr)) = query.get_mut(*entity) {
                                            *tr = new_local;
                                        }
                                    }
                                } else {
                                    // 부모가 없으면 로컬 == 월드 이므로 바로 설정
                                    if let Ok((_, mut tr)) = query.get_mut(*entity) {
                                        tr.translation = new_global_pos;
                                        tr.rotation = new_global_rot;
                                    }
                                }
                            } // for origin
                        } // if vectors ok
                    } // if pivot_2



                }
            }

            // 취소 처리
            if keyboard_input.just_pressed(KeyCode::Escape) || mouse_input.just_pressed(MouseButton::Right) {
                // 원래 상태로 복원
                for (parent, origin, translation) in origin.iter() {
                    let (_parent, mut transform) = query.get_mut(*parent)?;
                    transform.rotation = *origin;
                    transform.translation = *translation;
                }
                *control_status = MarkControlStatus::None;
            }
            // 확정 처리
            else if mouse_input.just_pressed(MouseButton::Left) || keyboard_input.just_pressed(KeyCode::Enter) {
                *control_status = MarkControlStatus::None;
            }
        },
    }

    Ok(())
}

fn handle_axis_locking(keyboard_input: &Res<ButtonInput<KeyCode>>, lock_mode: &mut LockAxis) {
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    if keyboard_input.just_pressed(KeyCode::KeyX) {
        if shift_pressed {
            // Shift + X: X축 제외하고 잠금
            *lock_mode = LockAxis::except_x();
        } else {
            // X: X축만 잠금
            *lock_mode = LockAxis::only_x();
        }
    }
    else if keyboard_input.just_pressed(KeyCode::KeyY) {
        if shift_pressed {
            // Shift + Y: Y축 제외하고 잠금
            *lock_mode = LockAxis::except_y();
        } else {
            // Y: Y축만 잠금
            *lock_mode = LockAxis::only_y();
        }
    }
    else if keyboard_input.just_pressed(KeyCode::KeyZ) {
        if shift_pressed {
            // Shift + Z: Z축 제외하고 잠금
            *lock_mode = LockAxis::except_z();
        } else {
            // Z: Z축만 잠금
            *lock_mode = LockAxis::only_z();
        }
    }
}

fn handle_rotation_axis(keyboard_input: &Res<ButtonInput<KeyCode>>, lock_mode: &mut RotationAxis) {
    let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    if keyboard_input.just_pressed(KeyCode::KeyX) {
        match lock_mode {
            RotationAxis::X(rmode) => match rmode {
                RotationMode::World if !shift_pressed => *lock_mode = RotationAxis::X(RotationMode::Local),
                RotationMode::LockWorld if shift_pressed => *lock_mode = RotationAxis::Y(RotationMode::LockLocal),
                _ => *lock_mode = RotationAxis::None
            }
            _ => {
                if !shift_pressed {
                    *lock_mode = RotationAxis::X(RotationMode::World)
                } else {
                    *lock_mode = RotationAxis::X(RotationMode::LockWorld)
                }
            },
        }
    }

}