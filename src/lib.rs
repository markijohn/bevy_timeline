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
use crate::data::{TimelineUntypedAnimation, TimelineUntypedResolver, TimelineUntypedTarget};
use crate::timeline::Keyframe;

// Timeline animation set
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AnimationSystemSet {
    /// Query [`TimelinePlayerLoader`] and load [`TimelineRawData`] from asset
    PreparePlayer,

    /// Each [`TimelineImplPlugin`] interprets the data and stores it in the [`TimelineResolvedCache`]
    ResolveType,

    /// [`TimelinePlayer`] sends `Event` via the saved session (entity, animation information)
    EventSender,

    /// Each [`TimelineImplPlugin`] receive the `Event` and dispatch
    TypedEventReceiver,
}

pub struct TimelinePlugin {
    resolver: HashMap<&'static str,TimelineUntypedResolver>,
}

impl TimelinePlugin {
    pub fn register_type<T:Component, A:Animatable>(mut self) -> Self {
        self.resolver.insert( A::typ(), TimelineUntypedResolver::new::<T,A>() );
        self
    }
}

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TimelineUntypedAnimation>()
            .register_asset_loader(TimelineRawDataLoader);
        app.configure_sets(
            PostUpdate,
            (
                AnimationSystemSet::PreparePlayer,
                AnimationSystemSet::ResolveType,
                AnimationSystemSet::EventSender,
                AnimationSystemSet::TypedEventReceiver,
            ).chain()
        );
        app
            .add_plugins( (
                TimelineImplPlugin::<Scale>::default(),
                TimelineImplPlugin::<Rotation>::default(),
                TimelineImplPlugin::<Translation>::default(),
            ) );
        app
            .add_systems(PostUpdate, load_player.in_set(AnimationSystemSet::PreparePlayer))

        ;
    }
}



fn load_player(
    mut cmds:Commands,
    timeline_data_assets: Res<Assets<TimelineUntypedAnimation>>,
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


fn send_anim_event<K>(
    players: Query<&mut TimelinePlayer>,
    step_writer: EventWriter<TimelineStepEvent<K>>,
) where K:AnimatableValue+Send+Sync+TypePath {

}

fn resolve_keyframes<K>(
    timeilne_assets: Res<Assets<TimelineRawData>>,
    resolve_assets: ResMut<Assets<Timeline<K>>>,
    mut players: Query<&mut TimelinePlayer, Added<TimelinePlayer>>,
    target_query: Query<&mut K::Target, With<Name>>,
) where K:AnimatableValue+Send+Sync+TypePath {
    //find exist resolved animation

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
// 
// 
// pub struct TimelineImplPlugin<K> where K:AnimatableValue + 'static {
//     inner : PhantomData<K>,
// }
// 
// impl <K> Default for TimelineImplPlugin<K> where K:AnimatableValue + 'static {
//     fn default() -> Self {
//         Self { inner : PhantomData }
//     }
// }
// 
// impl <K> Plugin for TimelineImplPlugin<K> where K:AnimatableValue + Send + Sync + TypePath + 'static {
// 
//     fn build(&self, app: &mut App) {
//         app
//             .insert_resouce(TimelineResolvedCache::<K>::default())
//             .add_event::<TimelineStepEvent<K>>()
//             .add_systems(PostUpdate, resolve_type::<K>.in_set(AnimationSystemSet::ResolveType))
//             .add_systems(PostUpdate, send_anim_event::<K>.in_set(AnimationSystemSet::EventSender))
//             .add_systems(PostUpdate, consume_step_event::<K>.in_set(AnimationSystemSet::TypedEventReceiver) );
//         ;
//     }
// }



// T = Transform
fn animate_interpol<T> (
    players: Query<&TimelinePlayer>,
    target: Query< &mut T >
) {
    for player in players.playing_sessions() {
        let entities = player.get_next_step_entities::<T>( );
        for (entity, targets:&TimelineUntypedAnimation) in entities {
            if let Ok(out_target) = target.get(entity) {
                targets.for_each( |anim| anim.interpolate() )
            }
        }
    }

}

fn type_animate<T:Component>(mut target_db: Query< &mut T >, list:Vec<(Entity,&[TimelineUntypedTarget])> ) {
    for (entity,target) in list {
        if let Ok(t) = target_db.get_mut(entity) {
            for target in targets {
                match target.typ {
                    A::typ() = > {

                        A::interpolate()
                    }
                }
            }
        }
    }
}



pub trait AnimatableSet {
    fn build(app:&mut bevy_app::App);
}

impl <V> AnimatableSet for V where V:AnimatableValue + 'static {
    fn build(app: &mut bevy_app::App) {
        app.add_plugins( TimelineImplPlugin::<V>::default() );
        app.add_systems( PostUpdate, Self::step_animation );
    }

}

impl <T,A,B> AnimatableSet for (A,B) where A:AnimatableValue<Target=T> + 'static, B:AnimatableValue<Target=T> + 'static  {
    fn build(app: &mut bevy_app::App) {
        
    }
}

fn test<T,A,B>(
    time: Res<Time>,
    query : Query<(&mut A::Target)>,
) where A:AnimatableValue + 'static, B:AnimatableValue + 'static {
    A::step_animation(players, query, )
}


macro_rules! impl_animatable_set {
    // 2개 요소 튜플
    ($($T:ident),+ $(,)?) => {
        impl_animatable_list!(@impl $($T),+);
    };

    // 실제 구현 생성
    (@impl $($T:ident),+) => {
        impl<$($T),+> AnimatableSet for ($($T,)+)
        where
            $($T: Animatable,)+
        {
            fn build(app:&mut bevy_app::App) {
                app.
            }

            fn animate_all(&self) {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                $(
                    $T.animate();
                )+
            }
        }
    };
}



macro_rules! impl_animatable_set_recursive {
    () => {};

    ($head:ident $(, $tail:ident)*) => {
        impl_animatable_set!($head $(, $tail)*);
        impl_animatable_set_recursive!($($tail),*);
    };
}

impl_animatable_set_recursive!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);




pub struct TimelineImplPlugin<K> where K:Component<Mutability=Mutable> {
    inner : PhantomData<K>,
}

impl <K> Default for TimelineImplPlugin<K> where K:Component<Mutability=Mutable> {
    fn default() -> Self {
        Self { inner : PhantomData }
    }
}

impl <K> Plugin for TimelineImplPlugin<K> where K:AnimatableValue + Send + Sync + TypePath + 'static {

    fn build(&self, app: &mut App) {
        app
            .insert_resouce(TimelineResolvedCache::<K>::default())
            .add_event::<TimelineStepEvent<K>>()
            .add_systems(PostUpdate, resolve_type::<K>.in_set(AnimationSystemSet::ResolveType))
            .add_systems(PostUpdate, send_anim_event::<K>.in_set(AnimationSystemSet::EventSender))
            .add_systems(PostUpdate, consume_step_event::<K>.in_set(AnimationSystemSet::TypedEventReceiver) );
        ;
    }
}
