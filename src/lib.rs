//! This is the crate that implements the animation of bevy.
//! 
//! It has nothing to do with the animations built into bevy.
//!
//! This library aims to be very accessible.
//! 
//! Timeline animations are available for the following types
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
struct TLInactive;

#[derive(Component)]
struct TLActive;

pub struct TimelinePlugin<Keyframe,Out> where Keyframe:AnimatableValue<Out>, Out: Component {
    inner : PhantomData<Keyframe>,
    inner_out : PhantomData<Out>,
}

impl <Keyframe,Out> Plugin for TimelinePlugin<Keyframe,Out> where Keyframe:AnimatableValue<Out> + Send + Sync + 'static, Out: Component {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, timeline_control_animation::<Keyframe,Out>.chain(timeline_step_animation::<Keyframe,Out>) )
        ;
    }
}



//start animation, stop animation
fn timeline_control_animation<T:AnimatableValue<Out>,Out:Component>(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline<T>>>,
    world: &mut World,
    players: Query<(Entity, &mut TimelinePlayer)>,
    childs: Query<(Entity, &Out, &TLInactive)>,
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


//interploate keyframe, stop animation
fn timeline_step_animation(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline>>,
    world: &mut World,
    players: Query<(Entity, &mut TimelinePlayer)>,
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