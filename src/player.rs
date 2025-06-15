use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::timeline::Timeline;

pub struct TimelinePlaySession {
    anim_handle : Handle<Timeline>,
    binded_targets : HashMap<Cow<'static,str>, Entity>
}

impl TimelinePlaySession {
    pub fn binded_targets(&self) -> impl Iterator<Item = (&Cow<'static, str>, &Entity)> {
        self.binded_targets.iter()
    }
}

#[derive(Component)]
pub struct TimelinePlayer {
    
    
}

impl TimelinePlayer {
    pub fn play_list(&self) -> &[TimelinePlaySession] {
        
    }
    
}