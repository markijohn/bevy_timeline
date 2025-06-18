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

use std::borrow::Cow;
use std::collections::HashMap;
pub use value::{AnimatableValue, TLTransform};
pub use player::{TimelinePlayer, TimelineSession};
pub use timeline::{Timeline};

use std::marker::PhantomData;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use bevy_transform::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::query::QueryData;
use bevy_reflect::TypePath;


// Timeline animation set
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AnimationSystemSet {
    Prepare,
    Update,
    Finalize,
}

#[derive(Component)]
struct TLActive;

#[derive(Component)]
struct TimelineStep( HashMap<Cow<'static,str>, f32> );

#[derive(Component)]
struct TimelineHandleCache<K>( HashMap<Cow<'static,str>, Handle<Timeline<K>>> );

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (
                AnimationSystemSet::Prepare,
                AnimationSystemSet::Update,
                AnimationSystemSet::Finalize,
            ).chain()
        );
        app
            .add_systems(PostUpdate, prepare_animation.in_set(AnimationSystemSet::Prepare))
            .add_systems(PostUpdate, consume_step::<TLTransform>.in_set(AnimationSystemSet::Update) )
            .add_systems(PostUpdate, finalize.in_set(AnimationSystemSet::Finalize))
        ;
    }
}

//Remove expired and stopped session
fn prepare_animation<K:AnimatableValue>(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline<K>>>,
    entity_players: Query<(&mut TimelinePlayer, Children)>,
    inactives: Query<(Entity, &mut K::Target), With<TLActive>>,
    actives: Query<(Entity, &mut K::Target), With<TLActive>>,
) where K:AnimatableValue+Send+Sync+TypePath {
    for player in entity_players {

    }
}

fn finalize(
    mut commands: Commands,
    players: Query<&mut TimelinePlayer>,
) {
    for mut player in players {
        player.reset_mark();
    }
}

// From here, the animation is interpolated and output to the actual animation target.
// Once the animation has ended, we remove the animation flag from the bound entity.
fn consume_step<K:AnimatableValue>(
    mut commands: Commands,
    assets: Res<Assets<Timeline<K>>>,
    active_anim_entities: Query<(Entity, &mut K::Target, &TimelineStep, &TimelineHandleCache<K>)>,
) where K:AnimatableValue+Send+Sync+TypePath {
    for (entity, out, step, cache) in active_anim_entities.iter() {
        for (anim_key, time) in step.iter() {
            let timeline = assets.get( cache.get(anim_key).unwrap() ).unwrap();
            timeline.interpolate( time, out );
            commands.remove::<TimelineStep>( entity );
        }
    }
    //Find plyaing session and enable timeline
    // for (entity, target) in inactives.iter() {
    //     for (_player_entity, player) in players.iter() {
    //         for (anim_name, session) in player.playing_sessions() {
    //             if let Some(find_my_entity) = session.binded_targets().iter().find( |e| **e == entity ) {
    //                 commands.entity(*find_my_entity).insert( TLActive );
    //             }
    //         }
    //     }
    // }
    // for (entity,player) in players.iter() {
    //     let anim_handle = player.play_list();
    //     if let Some(anim) = assets.get(anim_handle) {
    //         for (target, entity) in player.binded_targets() {
    //             let target = anim.get_target( target );
    //             if let Some(transform) = world.get_mut::<Transform>(*entity) {
    //                 //entity_mut.get::<Transform>();
    //             }
    //         }
    //     }
    // }
}

pub struct CustomTimelinePlugin<K> where K:AnimatableValue + 'static {
    inner : PhantomData<K>,
}

impl <K> Plugin for CustomTimelinePlugin<K> where K:AnimatableValue + Send + Sync + TypePath + 'static {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PostUpdate,
            (
                AnimationSystemSet::Prepare,
                AnimationSystemSet::Update,
                AnimationSystemSet::Finalize,
            ).chain()
        );
        app
            .add_systems(PostUpdate, consume_step::<K>.in_set(AnimationSystemSet::Update) );
        ;
    }
}
