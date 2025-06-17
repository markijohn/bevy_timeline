//! This is the crate that implements the animation of bevy.
//! 
//! It has nothing to do with the animations built into bevy.
//!
//! This project is in its early stages and is highly experimental.
//! 
//! Timeline animations are available for the following types
//! [`Transition`]: Changes the transition of a Transform.
//! [`Scale`]: Changes the scale of the Transform.
//! [`Rotation`]: Changes the rotation of the Transform.
//!
//! ## Implementation Notes
//! 
//! - The entity to be animated must have a [`TLAnimatable`] entity embedded in it.
//! - See `examples/custom_animatable` for custom animations

mod player;
mod timeline;
mod value;

use std::marker::PhantomData;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use bevy_transform::prelude::*;
use bevy_asset::prelude::*;
use crate::player::TimelinePlayer;
use crate::timeline::Timeline;
use crate::value::AnimatableValue;


#[derive(Component)]
struct TLActive;

pub struct TimelinePlugin<K> where K:AnimatableValue + 'static {
    inner : PhantomData<K>,
}

impl <K> Plugin for TimelinePlugin<K> where K:AnimatableValue + Send + Sync + 'static {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, timeline_control_animation::<K>.chain(timeline_step_animation::<K>) )
        ;
    }
}



//Here we check the player's state and insert the animation flag into the bound entity
fn timeline_control_animation<K:AnimatableValue>(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline<K>>>,
    world: &mut World,
    players: Query<(Entity, &mut TimelinePlayer)>,
    childs: Query<(Entity, &mut K::Target), Without(TLActive)>,
) {
    for (entity,player) in players.iter() {
        let anim_handle = player.play_list();
        if let Some(anim) = assets.get(anim_handle) {
            for (target, entity) in player.binded_targets() {
                let target = anim.get_target( target );
                if let Some(transform) = world.get_mut::<Transform>(*entity) {
                    //entity_mut.get::<Transform>();
                    //get frames and interpolate
                }
            }
        }
    }
}


// From here, the animation is interpolated and output to the actual animation target.
// Once the animation has ended, we remove the animation flag from the bound entity.
fn timeline_step_animation<K:AnimatableValue>(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline>>,
    players: Query<(Entity, &mut TimelinePlayer)>,
    inactives: Query<(Entity, &mut K::Target), With<TLActive>>,
    actives: Query<(Entity, &mut K::Target), With<TLActive>>,
) {

    for (entity,player) in players.iter() {
        let anim_handle = player.play_list();
        if let Some(anim) = assets.get(anim_handle) {
            for (target, entity) in player.binded_targets() {
                let target = anim.get_target( target );
                if let Some(transform) = world.get_mut::<Transform>(*entity) {
                    //entity_mut.get::<Transform>();
                }
            }
        }
    }
}