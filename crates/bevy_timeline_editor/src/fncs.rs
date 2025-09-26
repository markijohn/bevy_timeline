use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Commands, Entity, Event, Query, Resource, With};
use crate::UserSpawned;

#[derive(Resource)]
pub struct UtilityFns {
    pub clear_fn_id : SystemId,
}


pub struct UtilityFnsPlugin;

impl Plugin for UtilityFnsPlugin {
    fn build(&self, app: &mut App) {
        let clear_fn_id = app.world_mut().register_system( clear_user_spawned );
        app.insert_resource( UtilityFns { clear_fn_id } );
    }
}

fn clear_user_spawned(
    mut cmds:Commands,
    mut query: Query<Entity, With<UserSpawned>>
) {

    for e in query {
        cmds.entity(e).despawn();
    }
}