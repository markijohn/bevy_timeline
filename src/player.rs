use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::timeline::Timeline;

pub struct TimelineSession {
    is_loop : bool,
    is_loop_interpolation : bool,
    is_playing : bool,
    duration : f32,
    progress : f32,
    speed : f32,
    binded_targets : Vec<Entity>
}

impl TimelineSession {
    pub fn binded_targets(&self) -> impl Iterator<Item = (&Cow<'static, str>, &Entity)> {
        self.binded_targets.iter()
    }

    pub fn play(&mut self) {
        self.progress = 0.;
        self.is_playing = true;
    }

    pub fn resume(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }
}

#[derive(Component)]
pub struct TimelinePlayer {
    sessions : HashMap<Cow<'static,str>, TimelineSession>
}

impl TimelinePlayer {
    pub fn play(&mut self, name:&str) -> Option<bool> {
        if let Some(session) = self.sessions.get_mut(name) {
            let is_playing = session.is_playing;
            session.play();
            Some(is_playing)
        } else {
            None
        }
    }

    pub fn stop(&mut self, name:&str) -> Option<bool> {
        if let Some(session) = self.sessions.get_mut(name) {
            let is_playing = session.is_playing;
            session.stop();
            Some(is_playing)
        } else {
            None
        }
    }

    pub fn is_playing(&self) -> bool {
        for session in self.sessions.values() {
            if session.is_playing {
                return true
            }
        }
        false
    }

    pub fn stop_all(&mut self) {
        self.sessions.values_mut().for_each( |e| {
            e.stop();
        })
    }
    
}