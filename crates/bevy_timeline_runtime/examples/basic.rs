use std::ops::DerefMut;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use serde_json::Value;
use bevy_timeline_runtime::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TimelinePlugin::< (
            DefaultTransformSet,
        )>::new())
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, check)
        .run();
}


fn setup(
    mut asset_server: ResMut<AssetServer>,
    mut assets: Res<Assets<TimelineAnimation>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    
    // circular base
    commands.spawn((
        Name::new("Plane"),
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    
    // cube
    let mut player = TimelinePlayer::new().create_session( asset_server.load("basic.json#BasicTransform") );
    player.play("BasicTransform", TimelinePlayOption::new().set_playback(TimelinePlayback::ForwardLoop) );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        player,
    ));
    
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn check(
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    motion : Res<AccumulatedMouseMotion>,
    mut query: Single<(&Name, &Transform, &mut GlobalTransform)>,
) {
    if keycode.pressed(KeyCode::ArrowLeft) {

        let (name,tr, gtr) = query.deref_mut();
        println!("{tr:?} , {gtr:?}");
        // 현재 월드 위치에서 x를 1.5만큼 이동
        let mut affine = gtr.affine();
        affine.translation.x += 0.2;

        // GlobalTransform을 새로운 위치로 설정
        **gtr = GlobalTransform::from(affine);
    }
}