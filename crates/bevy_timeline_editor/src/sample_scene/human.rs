use bevy::prelude::*;


pub fn setup_human(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // // 조명 설정
    // commands.spawn(DirectionalLightBundle {
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });

    // // 바닥 평면
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Plane3d::default().mesh().size(10.0, 10.0)),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::srgb(0.3, 0.3, 0.3),
    //         ..default()
    //     }),
    //     transform: Transform::from_xyz(0.0, -0.1, 0.0),
    //     ..default()
    // });

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
            Name::new("Human"),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // 머리
    let head = commands
        .spawn((
            Mesh3d(head_mesh),
            Transform::from_xyz(0.0, 1.65, 0.0),
            MeshMaterial3d(material.clone()),
            Name::new("Head"),
        ))
        .id();

    // 몸통
    let torso = commands
        .spawn((
            Mesh3d(torso_mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 1.0, 0.0),
            Name::new("Torso"),
        ))
        .id();

    // 왼쪽 팔 (어깨)
    let left_arm = commands
        .spawn((
            Mesh3d(arm_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(-0.3, 1.3, 0.0),
            Name::new("Left_Arm"),
        ))
        .id();

    // 오른쪽 팔 (어깨)
    let right_arm = commands
        .spawn((
            Mesh3d(arm_mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.3, 1.3, 0.0),
            Name::new("Right_Arm"),
        ))
        .id();

    // 왼쪽 팔뚝
    let left_forearm = commands
        .spawn((
            Mesh3d(forearm_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(-0.3, 0.85, 0.0),
            Name::new("Left_Forearm"),
        ))
        .id();

    // 오른쪽 팔뚝
    let right_forearm = commands
        .spawn((
            Mesh3d(forearm_mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.3, 0.85, 0.0),
            Name::new("Right_Forearm"),
        ))
        .id();

    // 왼쪽 다리 (허벅지)
    let left_leg = commands
        .spawn((
            Mesh3d(leg_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(-0.1, 0.35, 0.0),
            Name::new("Left_Leg"),
        ))
        .id();

    // 오른쪽 다리 (허벅지)
    let right_leg = commands
        .spawn((
            Mesh3d(leg_mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.1, 0.35, 0.0),
            Name::new("Right_Leg"),
        ))
        .id();

    // 왼쪽 정강이
    let left_shin = commands
        .spawn((
            Mesh3d(shin_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(-0.1, -0.225, 0.0),
            Name::new("Left_Shin"),
        ))
        .id();

    // 오른쪽 정강이
    let right_shin = commands
        .spawn((
            Mesh3d(shin_mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.1, -0.225, 0.0),
            Name::new("Right_Shin"),
        ))
        .id();


    // 계층 구조 설정
    commands.entity(human_entity).add_children(&[
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
