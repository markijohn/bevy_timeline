use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::timeline::Timeline;
use crate::{TimelineAnimId, TimelineId, TimelineRawData};

#[derive(Default)]
pub enum TimelinePlayMode {
    #[default]
    ExactMatch,
    CropOrExtendEnd(f32),
    CropOrExtendStart(f32),
    Stretch(f32)
}


pub enum TimelineCursor {
    Rate(f32), // step = duration * rate
    Time(f32), // step = $param / duration
}

impl Default for TimelineCursor {
    fn default() -> Self {
        Self::Time(0.0)
    }
}

pub struct TimelineSession {
    is_loop : bool,
    is_loop_interpolation : bool,
    is_playing : bool,
    duration : f32,
    progress : f32,
    speed : f32,
    play_mode : TimelinePlayMode,
    timeline_db_id: AssetId<TimelineRawData>,
    binded_targets: HashMap<&'static str, Vec<(Entity,TimelineId)>>
}

impl TimelineSession {
    
    pub fn binded_targets(&self, typ:&'static str) -> &[(Entity,TimelineId)] {
        self.binded_targets.as_slice()
    }

    pub fn play(&mut self) {
        self.progress = 0.;
        self.resume();
    }

    pub fn resume(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }
    
    pub fn count_time(&mut self, elapsed:f32) {
        self.progress += elapsed;
    }
}

pub struct TimelinePlayOption {
    start: TimelineCursor,
    is_loop: bool,
    play_mode: TimelinePlayMode,
}

impl Default for TimelinePlayOption {
    fn default() -> Self {
        Self {
            start: TimelineCursor::Time(0f32),
            is_loop: false,
            play_mode: TimelinePlayMode::ExactMatch
        }
    }
}

#[derive(Component, Default)]
pub struct TimelinePlayer {
    unresolved_sessions: Vec<TimelineSession>,
    sessions : HashMap<TimelineAnimId, TimelineSession>,
}

impl TimelinePlayer {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn targets_mut(&mut self) -> &mut HashMap<Cow<'static,str>, Option<Entity>> {
        &mut self.binded_targets
    }


    pub fn create_session(&mut self, label:&'static str, handle:Handle<TimelineRawData>) {

    }
    
    pub fn sessions(&self) -> impl Iterator<Item=(&Cow<'static,str>, &TimelineSession)> {
        self.sessions.iter()
    }

    pub fn sessions_mut(&mut self) -> impl Iterator<Item=(&Cow<'static,str>, &mut TimelineSession)> {
        self.sessions.iter_mut()
    }
    
    pub fn play(&mut self, name:&str, option:Option<TimelinePlayOption>) -> Option<bool> {
        if let Some(session) = self.sessions.get_mut(name) {
            let is_playing = session.is_playing;
            session.play();
            Some(is_playing)
        } else {

        }
    }

    pub fn play(&mut self, name:&str, option:TimelinePlayOption) {

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