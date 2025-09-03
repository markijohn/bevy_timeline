use std::collections::HashMap;
use bevy::app::{App, Plugin};
use bevy::color::Color;
use bevy::ecs::system::SystemId;
use bevy::math::Vec3;
use bevy::prelude::{Children, Commands, Entity, Event, Name, PointLight, Query, ResMut, Resource, SpotLight, Transform, With, Without};
use bevy::text::{JustifyText, TextColor, TextFont, TextLayout};
use bevy_mod_billboard::prelude::*;

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


pub struct TimelineTargetListPlugin;

impl Plugin for TimelineTargetListPlugin {
    fn build(&self, app: &mut App) {
        let target_refresh = app.world_mut().register_system( make_treenode );
        app.insert_resource( TargetRefreshFn(target_refresh) );
        app.add_systems( bevy::prelude::Update, billboard_scaler_simple );
        app.insert_resource( TargetList { transform:Vec::new(), point_light:Vec::new() , spot_light : Vec::new() } );
    }
}

fn make_treenode(
    mut cmds : Commands,
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
                all_child_entities.insert(*child);
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
        entity: Entity,
        entity_map: &HashMap<Entity, (&Name, Option<&Children>)>
    ) -> Option<TreeNode> {
        if let Some((name, children_opt)) = entity_map.get(&entity) {
            let mut entity_cmd = cmds.entity(entity);
            entity_cmd.with_child( (
                BillboardText::new(name.as_str()),
                // TextFont::default().with_font_size(60.0),
                BillboardDepth(false),
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.0085)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));


            let mut tree_node = TreeNode {
                name: name.to_string(),
                entity,
                children: Vec::new(),
            };

            // 자식 엔티티들을 재귀적으로 처리
            if let Some(children) = children_opt {
                for child_entity in children.iter() {
                    if let Some(child_node) = build_tree_recursive(cmds, *child_entity, entity_map) {
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
        if let Some(root_node) = build_tree_recursive(&mut cmds,root_entity, &entity_map) {
            target_list.transform.push(root_node);
        }
    }
}

// 또는 더 간단한 버전 (고정 base_scale 사용)
fn billboard_scaler_simple(
    mut query: Query<&mut Transform, With<BillboardText>>,
    //camera_query: Query<&Transform, (With<bevy::prelude::Camera>, Without<BillboardText>)>,
    camera_query: bevy::prelude::Query<(&Transform,&bevy::prelude::Camera3d), Without<BillboardText>>,
) {
    const BASE_SCALE:f32 = 0.00085;

    if let Ok( (camera_transform,_) ) = camera_query.single() {
        for mut transform in query.iter_mut() {
            let distance = camera_transform.translation.distance(transform.translation);
            let scale_factor = distance * BASE_SCALE;
            transform.scale = Vec3::splat(scale_factor);
        }
    }
}