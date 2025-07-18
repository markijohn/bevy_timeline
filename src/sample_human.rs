use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_human, camera_controls))
        .run();
}

#[derive(Component)]
struct Human {
    animation_time: f32,
}

#[derive(Component)]
struct BodyPart {
    part_type: BodyPartType,
}

#[derive(Debug, Clone)]
enum BodyPartType {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftForearm,
    RightForearm,
    LeftLeg,
    RightLeg,
    LeftShin,
    RightShin,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 조명 설정
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // 카메라 설정
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    });

    // 바닥 평면
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10.0, 10.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -0.1, 0.0),
        ..default()
    });

    // 머티리얼 생성
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        ..default()
    });

    // 사람 모델 생성
    create_human(&mut commands, &mut meshes, body_material);
}

fn create_human(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
) {
    // 메시 생성
    let head_mesh = meshes.add(Sphere::new(0.15));
    let torso_mesh = meshes.add(Capsule3d::new(0.2, 0.8));
    let arm_mesh = meshes.add(Capsule3d::new(0.08, 0.4));
    let forearm_mesh = meshes.add(Capsule3d::new(0.06, 0.35));
    let leg_mesh = meshes.add(Capsule3d::new(0.1, 0.5));
    let shin_mesh = meshes.add(Capsule3d::new(0.08, 0.45));

    // 루트 엔티티 (전체 사람)
    let human_entity = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            Human {
                animation_time: 0.0,
            },
        ))
        .id();

    // 머리
    let head = commands
        .spawn((
            PbrBundle {
                mesh: head_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(0.0, 1.65, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::Head,
            },
        ))
        .id();

    // 몸통
    let torso = commands
        .spawn((
            PbrBundle {
                mesh: torso_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::Torso,
            },
        ))
        .id();

    // 왼쪽 팔 (어깨)
    let left_arm = commands
        .spawn((
            PbrBundle {
                mesh: arm_mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(-0.3, 1.3, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::LeftArm,
            },
        ))
        .id();

    // 오른쪽 팔 (어깨)
    let right_arm = commands
        .spawn((
            PbrBundle {
                mesh: arm_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(0.3, 1.3, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::RightArm,
            },
        ))
        .id();

    // 왼쪽 팔뚝
    let left_forearm = commands
        .spawn((
            PbrBundle {
                mesh: forearm_mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(-0.3, 0.85, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::LeftForearm,
            },
        ))
        .id();

    // 오른쪽 팔뚝
    let right_forearm = commands
        .spawn((
            PbrBundle {
                mesh: forearm_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(0.3, 0.85, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::RightForearm,
            },
        ))
        .id();

    // 왼쪽 다리 (허벅지)
    let left_leg = commands
        .spawn((
            PbrBundle {
                mesh: leg_mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(-0.1, 0.35, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::LeftLeg,
            },
        ))
        .id();

    // 오른쪽 다리 (허벅지)
    let right_leg = commands
        .spawn((
            PbrBundle {
                mesh: leg_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(0.1, 0.35, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::RightLeg,
            },
        ))
        .id();

    // 왼쪽 정강이
    let left_shin = commands
        .spawn((
            PbrBundle {
                mesh: shin_mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(-0.1, -0.225, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::LeftShin,
            },
        ))
        .id();

    // 오른쪽 정강이
    let right_shin = commands
        .spawn((
            PbrBundle {
                mesh: shin_mesh,
                material: material.clone(),
                transform: Transform::from_xyz(0.1, -0.225, 0.0),
                ..default()
            },
            BodyPart {
                part_type: BodyPartType::RightShin,
            },
        ))
        .id();

    // 계층 구조 설정
    commands.entity(human_entity).push_children(&[
        head,
        torso,
        left_arm,
        right_arm,
        left_forearm,
        right_forearm,
        left_leg,
        right_leg,
        left_shin,
        right_shin,
    ]);
}

fn animate_human(
    time: Res<Time>,
    mut human_query: Query<&mut Human>,
    mut body_parts: Query<(&mut Transform, &BodyPart), Without<Human>>,
) {
    for mut human in human_query.iter_mut() {
        human.animation_time += time.delta_seconds();

        let time_factor = human.animation_time;

        // 각 신체 부위별 애니메이션
        for (mut transform, body_part) in body_parts.iter_mut() {
            match body_part.part_type {
                BodyPartType::Head => {
                    // 머리 살짝 좌우로 흔들기
                    transform.rotation = Quat::from_rotation_y((time_factor * 0.5).sin() * 0.1);
                }
                BodyPartType::LeftArm => {
                    // 왼쪽 팔 스윙
                    transform.rotation = Quat::from_rotation_z((time_factor * 2.0).sin() * 0.3);
                }
                BodyPartType::RightArm => {
                    // 오른쪽 팔 스윙 (반대 방향)
                    transform.rotation = Quat::from_rotation_z(-(time_factor * 2.0).sin() * 0.3);
                }
                BodyPartType::LeftForearm => {
                    // 왼쪽 팔뚝 구부리기
                    transform.rotation = Quat::from_rotation_z((time_factor * 2.0).sin() * 0.2 + 0.3);
                }
                BodyPartType::RightForearm => {
                    // 오른쪽 팔뚝 구부리기
                    transform.rotation = Quat::from_rotation_z(-(time_factor * 2.0).sin() * 0.2 - 0.3);
                }
                BodyPartType::LeftLeg => {
                    // 왼쪽 다리 걷기 동작
                    transform.rotation = Quat::from_rotation_x((time_factor * 3.0).sin() * 0.4);
                }
                BodyPartType::RightLeg => {
                    // 오른쪽 다리 걷기 동작 (반대 방향)
                    transform.rotation = Quat::from_rotation_x(-(time_factor * 3.0).sin() * 0.4);
                }
                BodyPartType::LeftShin => {
                    // 왼쪽 정강이 구부리기
                    let bend = ((time_factor * 3.0).sin() * 0.5 + 0.5).max(0.0);
                    transform.rotation = Quat::from_rotation_x(bend * 0.8);
                }
                BodyPartType::RightShin => {
                    // 오른쪽 정강이 구부리기
                    let bend = (-(time_factor * 3.0).sin() * 0.5 + 0.5).max(0.0);
                    transform.rotation = Quat::from_rotation_x(bend * 0.8);
                }
                BodyPartType::Torso => {
                    // 몸통 살짝 좌우로 흔들기
                    transform.rotation = Quat::from_rotation_y((time_factor * 2.0).sin() * 0.05);
                }
            }
        }
    }
}

fn camera_controls(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    for mut transform in camera_query.iter_mut() {
        // 카메라를 인물 주위로 천천히 회전
        let radius = 8.0;
        let angle = time.elapsed_seconds() * 0.3;
        transform.translation.x = angle.cos() * radius;
        transform.translation.z = angle.sin() * radius;
        transform.look_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    }
}