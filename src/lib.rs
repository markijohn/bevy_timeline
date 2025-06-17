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
            .add_systems(PostUpdate, timeline_prepare.in_set(AnimationSystemSet::Prepare))
            .add_systems(PostUpdate, timeline_step::<TLTransform>.in_set(AnimationSystemSet::Update) )
            .add_systems(PostUpdate, timeline_finalize.in_set(AnimationSystemSet::Finalize))
        ;
    }
}


fn timeline_prepare() {
    
}

fn timeline_finalize() {
    
}

// From here, the animation is interpolated and output to the actual animation target.
// Once the animation has ended, we remove the animation flag from the bound entity.
fn timeline_step<K:AnimatableValue>(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline<K>>>,
    players: Query<(Entity, &mut TimelinePlayer)>,
    inactives: Query<(Entity, &mut K::Target), With<TLActive>>,
    actives: Query<(Entity, &mut K::Target), With<TLActive>>,
) where K:AnimatableValue+Send+Sync+TypePath {

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
            .add_systems(PostUpdate, timeline_step::<K>.in_set(AnimationSystemSet::Update) );
        ;
    }
}
