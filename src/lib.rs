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
//! - The entity to be animated must have a [`TimelinePlayerLoader`] entity embedded in it.
//! - See `examples/custom_animatable` for custom animations
//! 
//! ## Inner process
//! - After identifying the entity with the [`TimelinePlayerLoader`], it reads in the data from 
//! [`TimelineRawData`] and creates a hashtable so that TimeilneImplPlugins can quickly parse their type of data.
//! - The [`TimelineImplPlugin`] reads data that it can interpret from the type table in 
//! the [`TimelinePlayerLoader`] and stores the data in the [`TimelineResolvedCache`].

mod player;
mod timeline;
mod value;
mod loader;
mod data;

use std::any::TypeId;
use std::borrow::Cow;
use std::collections::HashMap;
pub use value::{AnimatableValue, Scale, Rotation, Translation};
pub use player::{TimelinePlayer, TimelineSession};
pub use timeline::{Timeline};

use std::marker::PhantomData;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::Time;
use bevy_transform::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::component::{ComponentMutability, Mutable};
use bevy_ecs::query::QueryData;
use bevy_reflect::TypePath;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use crate::data::{TimelineAnimation, TimelineAnimationSet, TimelineUntypedKeyframes, TimelineUntypedTarget};
use crate::loader::TimelineAnimationSetLoader;

pub type DefaultTransformSet = (Scale, Rotation, Translation);

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TimelineError {
    /// [IO Error](std::io::Error)
    #[error("JSON data load failed: {0}")]
    Io(#[from] std::io::Error),

    /// [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),

    #[error("Json value type is incorrect: {0}")]
    IncorrectValueType(&'static str),

    #[error("TimelineTarget cast failed(type not match) : {0} -> {1}")]
    TypeNotMatch{ request:&'static str, actual:&'static str },

    #[error("Unknown TimelineTarget type : {}")]
    UnknownTargetType(String),

    #[error("Unknown Timeline error : {0}")]
    UnknownError( Cow<'static,str> )
}

// Timeline animation set
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AnimationSystemSet {
    /// Binding [`Entity`] and [`TimelineUntypedTarget`]
    PreparePlayer,

    /// Handles animation processing from the [`TimelinePlayer`] entity and its children.
    Animate,
}

#[derive(Default)]
pub struct TimelinePlugin<B:TimelineImplSets> {
    inner: PhantomData<B>,
}


impl <B> Plugin for TimelinePlugin<B> where B:TimelineImplSets + Send + Sync + 'static {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<TimelineAnimationSet>()
            .init_asset::<TimelineAnimation>()
            .register_asset_loader( TimelineAnimationSetLoader );
        app.configure_sets(
            PostUpdate,
            (
                AnimationSystemSet::PreparePlayer,
                AnimationSystemSet::Animate,
            ).chain()
        );
        B::add_systems( app );
        app
            .add_systems(PostUpdate, bind_target_entities.in_set(AnimationSystemSet::PreparePlayer));
        ;
    }
}



fn bind_target_entities(
    mut player_loaders: Query<(Entity, &mut TimelinePlayer, Option<&Children>), Added<TimelinePlayer>>,
    childs: Query<(Entity, &Name, Option<&Children>)>,
) {
    // Collect entity paths: Vec<(Entity, Vec<&str>)>
    for (_self_entity, player, children) in player_loaders.iter_mut() {

        let mut entity_paths: Vec<(Entity, Vec<&str>)> = Vec::new();

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

        // entity_paths를 사용하여 필요한 작업 수행
        // ...
    }
}

fn load_player(
    mut cmds:Commands,
    timeline_data_assets: Res<Assets<TimelineAnimation>>,
    mut player_loaders: Query<(Entity, &mut TimelinePlayer, Option<&Children>), Added<TimelinePlayer> >,
    has_childs:Query<(Entity,&Name,&Children)>,
    last_level_childs:Query<(Entity,&Name), Without<Children>>,
) {
    //Collect named entity

    for (_self_entity, player_loader, children) in player_loaders.iter_mut() {
        let mut entity_map = HashMap::new();
        if let Some(children) = children {
            let mut dig = children.iter().collect::<Vec<Entity>>();
            while dig.len() > 0 {
                let Some(next) = dig.pop() else { break };
                if let Ok((dig_under, name, children)) = has_childs.get(next) {
                    //bind exist target
                    entity_map.insert( name.as_str(), dig_under.clone() );
                    dig.extend(children.iter().collect::<Vec<Entity>>());
                } else {
                    if let Ok((entity, name)) = last_level_childs.get(next) {
                        //bind exist target
                        entity_map.insert( name.as_str(), entity );
                    }
                }
            }
        }
        

        //Insert player
        for (id, req_anim_list) in player_loader.req_list.iter() {
            let mut player = TimelinePlayer::new();
            let (overwrite,player_mut) = if player_loader.overwrite {
                (true, exist_player.unwrap_or( &mut player ))
            } else {
                (false, &mut player)
            };
            if let Some(timeline_data) = timeline_data_assets.get( *id ) {
                for (anim_idx,anim) in timeline_data.anims.iter().enumerate() {
                    let mut session_bind_target:Vec<Option<(Entity,TimelineId)>> = Vec::new();
                    if req_anim_list.iter().find( |v| v.as_ref() == &anim.name ).is_some() {
                        let timeline_animid = TimelineAnimId { data_id: id.clone(), anim_idx };
                        for (target_idx,target) in anim.targets.iter().enumerate() {
                            let timeline_id = TimelineId { anim_id: timeline_animid.clone(), target_idx };
                            let target_name = target.name.as_str();
                            let bind = if target_name == "_self" {
                                Some( (_self_entity.clone(), timeline_id) )
                            } else if let Some(entity) = entity_map.get( target_name ) {
                                Some( (entity.clone(), timeline_id) )
                            } else {
                                // warn!("Can't find target {}({})", target_name, target.typ);
                                None
                            };
                            session_bind_target.push(bind);
                        }
                        player_mut.create_session(timeline_animid, session_bind_target);
                    }
                }

                if !overwrite {
                    cmds.entity(_self_entity)
                        .insert( player )
                        .remove::< TimelinePlayerLoader>();
                }

            }
        }
    }



}


fn animate_step<K>(
    mut cmds: Commands,
    anim_cache: Res<TimelineResolvedCache<K>>,
    keyframes: Res<Assets<Timeline<K>>>,
    players: Query<(&TimelinePlayer)>,
    target_query: Query<&mut K::Target, With<Name>>
) where K:AnimatableValue+Send+Sync+TypePath {
    for player in players.iter() {
        for (_name, session) in player.sessions().filter( | (_,s)| s.is_playing ) {
            for (target_entity, untyped_timeline_handle) = session.binded_targets() {
                if let Ok( target ) = target_query.get(target_entity) {
                    //get animatable
                    let timeline = untyped_timeline_handle.typed::<Timeline<K>>();
                    session.interpolate( timeline, target );
                }
            }
        }
    }
}

#[derive(Event)]
pub struct TimelineStepEvent<K> {
    entity: Entity,
    resovled_target_id: TimelineCacheId,
    inner: PhantomData<K>
}


fn consume_animation_step<K>(
    keyframes: Res<Assets<Timeline<K>>>,
    mut step_reader: EventReader<TimelineStepEvent<K>>,
    players: Query<(&TimelinePlayer)>,
    mut target_query: Query<&mut K::Target, With<Name>>
) where K:AnimatableValue+Send+Sync+TypePath {
    for event in step_reader.read() {
        if let Ok(mut target) = target_query.get_mut( event.entity ) {
            if let Some(timeline) = keyframes.get( event.resovled_target_id.typed_id::<Timeline<K>>() ) {
                timeline.interpolate( event.progress, target);
            }
        }
    }
}


//Remove expired and stopped session
//Mark step
fn prepare_animation<K:AnimatableValue>(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<Assets<Timeline<K>>>,
    entity_players: Query<(&mut TimelinePlayer, &Children)>,
    inactives: Query<(Entity, &mut K::Target)>,
    actives: Query<(Entity, &mut K::Target)>,
) where K:AnimatableValue+Send+Sync+TypePath {
    for player in entity_players {

    }
}


pub trait TimelineImplSets {
    fn add_systems(app:&mut App);
    
    fn try_resolve_keyframes(typ:&str, keyframes:&[Value]);
}


macro_rules! impl_timeline_impl_sets {
    ( $($tuple:ident),+ ) => {
        impl < $($tuple),+ > TimelineImplSets for ( $($tuple,)+ )
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


pub trait AnimatableSet {
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

impl <V> AnimatableSet for V where V:AnimatableValue + 'static {
    type Target = V::Target;

    fn step(
        assets: Res<Assets<TimelineAnimation>>,
        players: Query<&TimelinePlayer>,
        mut target_db: Query<&mut Self::Target>,
    ) {
        for player in players {
            for session in player.sessions().filter( |s| s.is_playing() ) {
                if let Some(timeline) = assets.get( &session.anim_handle ) {
                    for binded_target in session.get_entities::<V>() {
                        if let Ok(out) = target_db.get_mut(binded_target.entity) {
                            if let Ok(keyframes) = timeline.targets[ binded_target.target_idx ].get_typed::<V>() {
                                V::interpolate_from_keyframe(session.progress, session.progress, keyframes, out);
                            }
                        }
                    }
                }
            }
        }
    }

    fn try_resolve_keyframes(typ:&str, value:&[Value]) -> Option<Result<TimelineUntypedKeyframes,TimelineError>> {
        if typ == V::typ() {
            Some( V::to_untyped_keyframes( value ) )
        } else {
            None
        }
    }
}

macro_rules! impl_animatable_set {
    ( $F:ident, $($T:ident),+ ) => {
        impl<$F, $($T),+> AnimatableSet for ($F, $($T,)+)
        where
            $F:AnimatableValue, $($T: AnimatableValue,)+
        {
            type Target = $F::Target;

            fn try_resolve_keyframes(typ:&str, value:&[Value]) -> Option<Result<TimelineUntypedKeyframes,TimelineError>> {
                if $F::typ() == typ {
                    return $F::try_resolve_keyframes( typ, value )
                }
                $(
                if $T::typ() == typ {
                    return $F::try_resolve_keyframes( typ, value )
                }
                )+
                None
            }
        }
    };
}

// impl_animatable_set!(T1);
// impl_animatable_set!(T1, T2);
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