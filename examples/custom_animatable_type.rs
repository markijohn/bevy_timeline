use std::borrow::Cow;
use bevy::prelude::*;
use serde_json::Value;
use bevy_timeline::{AnimatableValue, TimelinePlayer, TimelinePlugin};


struct PointLightIntensity(f32);
impl AnimatableValue for PointLightIntensity {
    type Target = PointLight;

    fn interpolate(s: f32, start: Self, end: Self, out: &mut Self::Target) {
        out.intensity = start.0.lerp( end.0, s );
    }

    fn from_value(value: &Value) -> std::result::Result<Self, Cow<'static, str>> {
        Ok( Self( value.as_number().unwrap().as_f64().unwrap() as f32 ) )
    }

    fn to_value(&self) -> Value {
        Value::Number( serde_json::Number::from(self.0) )
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            TimelinePlugin::default().register_set( (PointLightIntensity, PointLightRange, PointLightRadius) )
        )
        .add_systems(Startup, setup)
        .run();
}


fn setup(
    mut commands: Commands,
    mut assets_server: Res<AssetServer>,
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
    let cube = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        TimelinePlayer::new(),
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
    let mut player = TimelinePlayer::from( assets_server.load("custom_") );
    player.play_as("some_anim1", TimelinePlayMode.set_loop(true) );
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        PointLight,
        player
    ));

}