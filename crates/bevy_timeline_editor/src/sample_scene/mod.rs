use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemId;
use bevy::prelude::{Event, Resource};

mod shapes;
mod human;

#[derive(Resource)]
pub struct SpawnSampleScene {
    pub shapes_fn_id : SystemId,
    pub human_fn_id : SystemId,
}


pub struct SampleScenePlugin;

impl Plugin for SampleScenePlugin {
    fn build(&self, app: &mut App) {
        let shapes_fn_id = app.world_mut().register_system( shapes::setup_shapes );
        let human_fn_id = app.world_mut().register_system( human::setup_human );
        app.insert_resource( SpawnSampleScene { shapes_fn_id, human_fn_id } );
    }
}