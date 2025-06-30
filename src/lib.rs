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
use bevy_ecs::query::QueryData;
use bevy_reflect::TypePath;
use serde::Deserialize;
use serde_json::Value;
use crate::data::TimelineRawData;
use crate::loader::TimelineRawDataLoader;
use crate::timeline::Keyframe;

#[derive(Component)]
pub struct TimelinePlayerLoader {
    //if this value is `true` then keep [`TimelinePlayer`] in current entity and overwrite session. default is `false`
    overwrite: bool,
    
    //user create from `load`, `load_all`
    //timeline_db_id : import anima names
    req_list: HashMap<AssetId<TimelineRawData>, Vec<Cow<'static,str>>>,

    //This is just an intermediate step to minimize search duplication.
    //inner cache. separate as type
    //typ : (timeline_db_id, anim_idx, target_idx)
    type_list: HashMap<String, Vec< TimelineId > >,
}

impl TimelinePlayerLoader {
    pub fn new() -> Self {
        Self {
            overwrite: false,
            req_list: HashMap::new(),
            type_list: HashMap::new(),
        }
    }

    pub fn load_all(self, data:AssetId<TimelineRawData>) -> Self {
        self.load(data, vec![])
    }

    pub fn load(mut self, data:AssetId<TimelineRawData>, required_anims:Vec<Cow<'static,str>>) -> Self {
        if let Some(exist) = self.req_list.get_mut( &data ) {
            exist.extend( required_anims );
        } else {
            self.req_list.insert(data, required_anims);
        }

        self
    }

}

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct TimelineAnimId {
    pub data_id: AssetId<TimelineRawData>,
    pub anim_idx: usize,
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct TimelineId {
    pub anim_id: TimelineAnimId,
    pub target_idx: usize,
}

impl TimelineId {
    pub fn from_data<A:AsRef<str>,B:AsRef<str>>(data:&TimelineRawData, handle:AssetId<TimelineRawData>, anim_name:A, typ:&'static str, target_name:&str) -> Option<Self> {
        let (anim_idx, anim) = data.anims.iter().enumerate().find( |(idx, anim)| anim.name == anim_name.as_ref() )?;
        let (target_idx, _target) = anim.targets.iter().enumerate().find( |(idx, target)| target.name == target_name )?;
        Some( Self {
            anim_id: TimelineAnimId { 
                data_id: handle,
                anim_idx,
            },
            target_idx
        } )
    }
}

#[derive(Default, Resource)]
struct TimelineResolvedCache<K> where K:AnimatableValue+Send+Sync{
    cache: HashMap< TimelineId, Timeline<K>>
}


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

#[derive(Component)]
pub struct TimelineTargetRebind;



pub struct TimelinePlugin {
    resolver: HashMap<&'static str,TimelineUntypedResolver>,
}

impl TimelinePlugin {
    pub fn register_type<T:AnimatableValue>(mut self) -> Self {
        self.resolver.insert( T::typ(), )
        self
    }
}

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TimelineRawData>()
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
    timeline_data_assets: Res<Assets<TimelineRawData>>,
    mut player_loaders: Query<(Entity, &mut TimelinePlayerLoader, Option<&TimelinePlayer>, Option<&Children>), Added<TimelinePlayerLoader> >,
    has_childs:Query<(Entity,&Name,&Children)>,
    last_level_childs:Query<(Entity,&Name), Without<Children>>,
) {
    //Collect named entity

    for (_self_entity, player_loader, exist_player, children) in player_loaders.iter_mut() {
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

fn finalize(
    mut commands: Commands,
    players: Query<&mut TimelinePlayer>,
) {
    for mut player in players {
        player.reset_mark();
    }
}


pub struct TimelineImplPlugin<K> where K:AnimatableValue + 'static {
    inner : PhantomData<K>,
}

impl <K> Default for TimelineImplPlugin<K> where K:AnimatableValue + 'static {
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




fn resolve_type<K>(
    timeline_data_assets: Res<Assets<TimelineRawData>>,
    timeline_resolved: ResMut<TimelineResolvedCache<K>>,
    player_loaders_query: Query<&TimelinePlayerLoader>
) where K:AnimatableValue+Send+Sync+TypePath {
    for player in player_loaders_query {
        if let Some( tid ) = player.type_list.get( K::typ() ) {
            let is_exist = timeline_resolved.cache.get( &tid ).is_some();
            if !is_exist {

            }
            let timeline_data = timeline_data_assets.get( *id ).unwrap(); //unreachable
            let timeline_target = &timeline_data.anims[*anim_idx].targets[*target_idx];


            let timeline = match timeline_target.to_timeline::<K>() {
                Ok(timeline) => {
                    timeline
                },
                Err(e) => {
                    //todo : handle error
                    panic!( "timeline type({}) resolve failed : {}", K::typ(), e)
                }
            };
            let id = TimelineId {
                data_handle: id.clone(),
                typ: K::typ(),
                anim_idx: anim_idx,
                target_idx: target_idx,
            };

        }
    }
}

// From here, the animation is interpolated and output to the actual animation target.
// Once the animation has ended, we remove the animation flag from the bound entity.

fn consume_step_event<K>(
    timeline_data_assets: Res<Assets<TimelineRawData>>,
    player_loaders_query: Query<&TimelinePlayer>,

) where K:AnimatableValue+Send+Sync+TypePath {

}