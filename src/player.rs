use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::timeline::Timeline;


pub struct TimelineSession {
    is_loop : bool,
    is_loop_interpolation : bool,
    mark_changed : bool,
    is_playing : bool,
    duration : f32,
    progress : f32,
    speed : f32,
    
    binded_targets : Vec<Entity>
}

impl TimelineSession {
    pub fn binded_targets(&self) -> &[Entity] {
        self.binded_targets.as_slice()
    }

    pub fn play(&mut self) {
        self.progress = 0.;
        self.resume();
    }

    pub fn resume(&mut self) {
        if !self.is_playing {
            self.mark_changed = true;
        }
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        if self.is_playing {
            self.mark_changed = true;
        }
        self.is_playing = false;
    }

    pub fn reset_mark(&mut self) {
        self.mark_changed = false;
    }
}

#[derive(Component)]
pub struct TimelinePlayer {
    sessions : HashMap<Cow<'static,str>, TimelineSession>
}

impl TimelinePlayer {
    pub fn playing_sessions(&self, mark_changed:Option<bool>) -> impl Iterator<Item=(&Cow<'static,str>, &TimelineSession)> {
        self.sessions.iter().filter( | (_,s)| {
            if let Some(flag) = mark_changed {
                s.mark_changed == flag && s.is_playing
            } else {
                s.is_playing
            }
        })
    }

    pub fn stoped_sessions(&self, mark_changed:Option<bool>) -> impl Iterator<Item=(&Cow<'static,str>, &TimelineSession)> {
        self.sessions.iter().filter( | (_,s)| {
            if let Some(flag) = mark_changed {
                s.mark_changed == flag && !s.is_playing
            } else {
                !s.is_playing
            }
        })
    }

    pub fn sessions(&self) -> impl Iterator<Item=(&Cow<'static,str>, &TimelineSession)> {
        self.sessions.iter()
    }
    
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

    pub fn reset_mark(&mut self) {
        self.sessions.values_mut().for_each( |e| {
            e.reset_mark();
        })
    }
}