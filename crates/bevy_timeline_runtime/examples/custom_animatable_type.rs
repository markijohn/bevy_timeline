use bevy::prelude::*;
use serde::{Serialize,Deserialize};
use bevy_timeline_runtime::prelude::*;


#[derive(Default, Clone, Serialize, Deserialize)]
struct PointLightIntensity(f32);

impl AnimatableValue for PointLightIntensity {
    type Target = PointLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.intensity = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct PointLightRadius(f32);

impl AnimatableValue for PointLightRadius {
    type Target = PointLight;

    fn interpolate(s: f32, start: Self, end: Option<Self>, out: &mut Self::Target) {
        out.radius = start.0.lerp( end.unwrap_or( Self(0.) ).0, s  );
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TimelinePlugin::< (
            DefaultTransformSet,
            (PointLightIntensity, PointLightRadius)
        )>::new())
        .add_systems(Startup, setup)
        .run();
}


fn setup(
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // cube
    let mut player = TimelinePlayer::new().create_session( asset_server.load("custom.json#BasicTransform") );
    player.play("BasicTransform", TimelinePlayOption::new().set_playback(TimelinePlayback::ForwardLoop) );
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        player,
    ));

    // light
    let mut player = TimelinePlayer::new().create_session( asset_server.load("custom.json#PointLightAnim") );
    player.play("PointLightAnim", TimelinePlayOption::new().set_playback(TimelinePlayback::ForwardLoop) );
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        player,
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

}