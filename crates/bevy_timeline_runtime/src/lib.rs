//! This is the crate that implements the animation of bevy.
//! 
//! It has nothing to do with the animations system built into bevy.
//!
//! This project is in its early stages and is highly experimental.
//! 
//!
//! ## Implementation Notes
//! 
//! - The entity to be animated must have a [`TimelinePlayer`] entity embedded in it.
//! - See `bevy_timeline_runtime/examples/custom_animatable_type` for custom animations
//! - `bevy_timeline_runtime` is a library that can run animations.
//! - `bevy_timeline_impls` is a set that implements animation interpolation behavior and is not a required element.
//! - `bevy_timeline_editor` is an editor that can create and edit animations by applying elements in `bevy_timeline_impls`.

mod player;
pub mod value;
mod loader;
mod data;

use std::borrow::Cow;

use value::{AnimatableValue, Scale, Rotation, Translation};
use player::{TimelinePlayer};

use std::marker::PhantomData;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use bevy_asset::prelude::*;
use bevy_ecs::component::{Mutable};
use serde_json::Value;
use thiserror::Error;
use crate::data::{TimelineAnimation, TimelineAnimationSet, TimelineUntypedKeyframes};
use crate::loader::TimelineAnimationSetLoader;
use crate::player::TimelineTargetBinded;

pub type DefaultTransformSet = (Scale, Rotation, Translation);

pub mod prelude {
    pub use crate::{TimelinePlugin, TimelineError, AnimatableSet, TimelineImplSets};
    pub use crate::DefaultTransformSet;
    pub use crate::player::{TimelinePlayer, TimelineSession, TimelinePlayback, TimelinePlayOption};
    pub use crate::value::{AnimatableValue,ValueExt};
    pub use crate::data::{TimelineAnimation, TimelineAnimationSet, TimelineUntypedKeyframes, TimelineUntypedTarget};
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TimelineError {
    /// [IO Error](std::io::Error)
    #[error("JSON data load failed: {0}")]
    Io(#[from] std::io::Error),

    /// [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),

    
    #[error("Could not find item: {0}")]
    NotExistItem(&'static str),

    #[error("array length not match(required:{required}) but {actual}")]
    InvalidLength{required:usize, actual:usize},

    #[error("Json value type is incorrect: {0}")]
    IncorrectValueType(&'static str),

    #[error("TimelineTarget cast failed(type not match) : {request} -> {actual}")]
    TypeNotMatch{ request:&'static str, actual:&'static str },
    

    #[error("Unknown TimelineTarget type : {0}")]
    UnknownTargetType(String),

    #[error("Unknown Timeline error : {0}")]
    UnknownError( Cow<'static,str> )
}

// Timeline animation set
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AnimationSystemSet {
    /// Binding [`Entity`] and [`TimelineUntypedTarget`]
    PreparePlayer,

    /// Update time
    Update,

    /// Handles animation processing from the [`TimelinePlayer`] entity and its children.
    Animate,
}

#[derive(Default)]
pub struct TimelinePlugin<B:TimelineImplSets> {
    inner: PhantomData<B>,
}

impl <B:TimelineImplSets> TimelinePlugin<B> {
    pub fn new() -> Self{
        Self {
            inner: PhantomData,
        }
    }
}


impl <B> Plugin for TimelinePlugin<B> where B:TimelineImplSets + Send + Sync + 'static {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<TimelineAnimationSet>()
            .init_asset::<TimelineAnimation>()
            .register_asset_loader( TimelineAnimationSetLoader::<B>::new() );
        app.configure_sets(
            PostUpdate,
            (
                AnimationSystemSet::PreparePlayer,
                AnimationSystemSet::Update,
                AnimationSystemSet::Animate,
            ).chain()
        );
        B::add_systems( app );
        app
            .add_systems(PostUpdate, bind_target_entities.in_set(AnimationSystemSet::PreparePlayer))
            .add_systems(PostUpdate, update_time.in_set(AnimationSystemSet::Update))
        ;
    }
}

fn update_time(
    mut time: Res<Time>,
    mut player_loaders: Query<&mut TimelinePlayer>,
) {
    for mut timeline_player in player_loaders.iter_mut() {
        timeline_player.update_time( time.delta_secs() );
    }
}

fn bind_target_entities(
    assets: Res<Assets<TimelineAnimation>>,
    mut player_loaders: Query<(Entity, &mut TimelinePlayer, Option<&Children>), Added<TimelinePlayer>>,
    // mut player_loaders: Query<(Entity, &mut TimelinePlayer, Option<&Children>)>,
    childs: Query<(Entity, &Name, Option<&Children>)>,
) {
    // Collect entity paths: Vec<(Entity, Vec<&str>)>
    for (_self_entity, mut player, children) in player_loaders.iter_mut() {
        let mut entity_paths: Vec<(Entity, Vec<&str>)> = Vec::new();
        entity_paths.push( (_self_entity, vec!["_self"]) );

        let sessions = player.sessions_mut();

        if let Some(children) = children {
            //entity & path to parent
            let mut stack: Vec<(Entity, Vec<&str>)> = Vec::new();

            //add to stack
            for child in children.iter() {
                stack.push((child, Vec::new()));
            }

            while let Some((current_entity, parent_path)) = stack.pop() {
                if let Ok((entity, name, children)) = childs.get(current_entity) {
                    // 현재 노드까지의 경로 생성
                    let mut current_path = parent_path.clone();
                    current_path.push(name.as_str());

                    // 결과에 추가
                    entity_paths.push((entity, current_path.clone()));

                    // 자식들을 스택에 추가
                    if let Some(in_children) = children {
                        for child in in_children.iter() {
                            stack.push((child, current_path.clone()));
                        }
                    }
                }
            }
        }

        // 결과 출력 (디버깅용)
        for (entity, path) in &entity_paths {
            println!("Entity {:?}: {:?}", entity, path);
        }

        let mut short_cuts = vec![];
        for session in sessions {
            println!("Session: {:?}", session.anim_handle);
            let name = if let Some(timeline) = assets.get( &session.anim_handle ) {
                let mut binded_targets = Vec::new();
                for (target_idx,target) in timeline.targets.iter().enumerate() {
                    let names = target.target.as_slice();
                    if let Some( finded) = entity_paths.iter().find( |e| e.1.as_slice() == names) {
                        let binded = TimelineTargetBinded {
                            typ : target.typ(),
                            entity: finded.0,
                            target_idx
                        };
                        binded_targets.push( binded );
                    }
                }
                session.duration = timeline.duration;
                session.set_binded( binded_targets );
                println!("Some binded");
                timeline.name.clone()
            } else {
                "".to_string()
            };
            short_cuts.push( name );
        }
        player.set_shortcuts( short_cuts );

        // entity_paths를 사용하여 필요한 작업 수행
        // ...
    }
}


pub trait TimelineImplSets {
    fn add_systems(app:&mut App);
    
    fn try_resolve_keyframes(typ:&str, keyframes:&[Value]) -> Option<Result<TimelineUntypedKeyframes,TimelineError>>;
}


macro_rules! impl_timeline_impl_sets {
    ( $($tuple:ident),+ ) => {
        impl < $($tuple,)+ > TimelineImplSets for ( $($tuple,)+ )
        where $($tuple:AnimatableSet,) +
        {
            fn add_systems(app:&mut App) {
                $(
                <$tuple as AnimatableSet>::add_system( app );
                )+
            }

            fn try_resolve_keyframes(typ:&str, keyframes:&[Value]) -> Option<Result<TimelineUntypedKeyframes,TimelineError>> {
                $(
                let result = <$tuple as AnimatableSet>::try_resolve_keyframes( typ, keyframes );
                if result.is_some() {
                    return result;
                }
                )+
                None
            }
        }
    };
}

impl_timeline_impl_sets!( T1 );
impl_timeline_impl_sets!( T1, T2 );
impl_timeline_impl_sets!( T1, T2, T3 );
impl_timeline_impl_sets!( T1, T2, T3, T4 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 );
impl_timeline_impl_sets!( T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 );


pub trait AnimatableSet where Self: 'static {
    type Target:Component<Mutability=Mutable>;
    fn add_system(app:&mut bevy_app::App) {
        app.add_systems( PostUpdate, Self::step.in_set(AnimationSystemSet::Animate) );
    }

    fn step(
        assets: Res<Assets<TimelineAnimation>>,
        players: Query<&TimelinePlayer>,
        target_db: Query<&mut Self::Target>,
    );

    fn try_resolve_keyframes(typ:&str, value:&[Value]) -> Option<Result<TimelineUntypedKeyframes,TimelineError>>;
}

macro_rules! impl_animatable_set {
    ( $F:ident, $($T:ident),* ) => {
        impl<$F, $($T),*> AnimatableSet for ($F, $($T,)*)
        where
            $F:AnimatableValue, $($T: AnimatableValue<Target=$F::Target>,)*
        {
            type Target = $F::Target;

            fn step(
                assets: Res<Assets<TimelineAnimation>>,
                players: bevy_ecs::system::Query<&TimelinePlayer>,
                mut outputs: bevy_ecs::system::Query<&mut <Self as AnimatableSet>::Target>) {

                for player in players {
                    for session in player.sessions().filter( |s| s.is_playing() ) {
                        if let Some(timeline) = assets.get( &session.anim_handle ) {
                            for binded_target in session.get_entities::<$F>() {
                                if let Ok(out) = outputs.get_mut(binded_target.entity) {
                                    if let Ok(keyframes) = timeline.targets[ binded_target.target_idx ].get_typed::<$F>() {
                                        $F::interpolate_from_keyframe(session.duration, session.prev_progress, session.progress, keyframes, out);
                                    }
                                }
                            }

                            $(
                            for binded_target in session.get_entities::<$T>() {
                                if let Ok(out) = outputs.get_mut(binded_target.entity) {
                                    if let Ok(keyframes) = timeline.targets[ binded_target.target_idx ].get_typed::<$T>() {
                                        //println!("interpol {} {} {}", session.duration, session.prev_progress, session.progress);
                                        $T::interpolate_from_keyframe(session.duration, session.prev_progress, session.progress, keyframes, out);
                                    }
                                }
                            }
                            )*
                        }
                    }
                }
            }

            fn try_resolve_keyframes(typ:&str, value:&[Value]) -> Option<Result<TimelineUntypedKeyframes,TimelineError>> {
                if $F::typ() == typ {
                    return Some( $F::create_untyped_keyframes( value ) )
                }
                $(
                if $T::typ() == typ {
                    return Some( $T::create_untyped_keyframes( value ) )
                }
                )*
                None
            }

        }
    };
}

impl_animatable_set!(T1,);
impl_animatable_set!(T1, T2);
impl_animatable_set!(T1, T2, T3);
impl_animatable_set!(T1, T2, T3, T4);
impl_animatable_set!(T1, T2, T3, T4, T5);
impl_animatable_set!(T1, T2, T3, T4, T5, T6);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_animatable_set!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);