use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::timeline::Timeline;
use crate::TimelineRawData;


pub struct TimelineAnimation {
    pub is_loop : bool,
    pub is_loop_interpolation : bool,
    pub mark_changed : bool,
    pub is_playing : bool,
    pub duration : f32,
    pub progress : f32,
    pub speed : f32,
}

impl TimelineAnimation {
    
    pub fn binded_targets(&self) -> &[(Entity,UntypedHandle)] {
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
    
    pub fn count_time(&mut self, elapsed:f32) {
        self.progress += elapsed;
    }

    pub fn reset_mark(&mut self) {
        self.mark_changed = false;
    }
}

#[derive(Component, Default)]
pub struct TimelinePlayer {
    sessions : HashMap<Cow<'static,str>, TimelineSession>,
    binded_targets : HashMap<Cow<'static,str>, Option<Entity>>,
}

impl TimelinePlayer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn targets_mut(&mut self) -> &mut HashMap<Cow<'static,str>, Option<Entity>> {
        &mut self.binded_targets
    }

    pub fn load_all(self, data:Handle<TimelineRawData>) -> Self {
        self.load(data, None)
    }

    pub fn load(self, data:Handle<TimelineRawData>, import_anims:Option<&[&str]>) -> Self {
        for (name, anim) in data.anims.iter() {
            if import_anims.is_none() ||
                import_anims.unwrap().iter().find( |s| **s == name.as_str() ).is_some() {

            }
        }
        self
    }

    pub fn create_session(&mut self, label:&'static str, handle:Handle<TimelineRawData>) {

    }
    
    pub fn sessions(&self) -> impl Iterator<Item=(&Cow<'static,str>, &TimelineSession)> {
        self.sessions.iter()
    }

    pub fn sessions_mut(&mut self) -> impl Iterator<Item=(&Cow<'static,str>, &mut TimelineSession)> {
        self.sessions.iter_mut()
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