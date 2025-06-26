use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::timeline::Timeline;
use crate::{TimelineAnimId, TimelineId, TimelineRawData};

#[derive(Default)]
pub enum TimelinePlayMode {
    /// Run with default duration
    #[default]
    ExactMatch,
    
    /// Runs in the given time and if it is less than the animation time, it will be cut off at that time or extended at the end if it
    CropOrExtendEnd(f32),
    
    /// If it runs in the given time and is less than the animation time, the difference is set to the start time of the playback, and if longer, it is extended from the beginning
    CropOrExtendStart(f32),
    
    /// It runs in a given amount of time and is incremented or decremented, i.e. `1.0` will play at the default playback speed and if you set it to `0.5` it will play twice as fast.
    Stretch(f32)
}

pub enum TimelineCursor {
    /// Relative start time
    Rate(f32), // step = duration * rate
    
    /// Absolute start time
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
    timeline_anim_id: TimelineAnimId,
    binded_targets: Vec<Option<(Entity,TimelineId)>>
}

impl TimelineSession {
    /// binded child (entity,timeline)
    pub fn binded_targets(&self) -> &[Option<(Entity,TimelineId)>] {
        self.binded_targets.as_slice()
    }

    /// The `play` function always starts at time `0` (but can depend on the start of `TimelinePlayOption`)
    pub fn play(&mut self, option:TimelinePlayOption) {
        self.progress = 0.;
        if let Some(start) = option.start {
            match start {
                TimelineCursor::Rate(r) => { self.progress = self.duration * r }
                TimelineCursor::Time(t) => { self.progress = t }
            }
        }
        if let Some(is_loop) = option.is_loop {
            self.is_loop = is_loop;
        }
        if let Some(play_mode) = option.play_mode {
            self.play_mode = play_mode;
        }
        self.resume();
    }

    /// Resume from the point where `stop` was called
    pub fn resume(&mut self) {
        self.is_playing = true;
    }

    /// Pauses the animation playing in the current session
    pub fn stop(&mut self) {
        self.is_playing = false;
    }
    
    /// Accumulate playback time
    pub fn update_time(&mut self, elapsed:f32) {
        self.progress += elapsed;
    }
}

#[derive(Default)]
pub struct TimelinePlayOption {
    start: Option<TimelineCursor>,
    is_loop: Option<bool>,
    play_mode: Option<TimelinePlayMode>,
}

impl TimelinePlayOption {
    /// Set the playback start time
    pub fn start_at(mut self, start:TimelineCursor) -> Self{
        self.start = Some(start);
        self
    }
    
    /// Set loop mode
    pub fn set_loop(mut self, is_loop:bool) -> Self{
        self.is_loop = Some(is_loop);
        self
    }
    
    /// Set play mode
    pub fn set_play_mode(mut self, play_mode:TimelinePlayMode) -> Self{
        self.play_mode = Some(play_mode);
        self
    }
}



#[derive(Component, Default)]
pub struct TimelinePlayer {
    sessions : Vec<TimelineSession>,
    shortcut : HashMap<String, usize>
}

impl TimelinePlayer {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn create_session(&mut self, timeline_anim_id:TimelineAnimId, binded_targets:Vec<Option<(Entity,TimelineId)>>) {
        self.sessions.push(TimelineSession {
            is_loop: false,
            is_loop_interpolation: true,
            is_playing: false,
            duration: 0.0,
            progress: 0.0,
            speed: 1.0,
            play_mode: Default::default(),
            timeline_anim_id,
            binded_targets,
        });
    }
    
    pub fn sessions(&self) -> impl Iterator<Item=&TimelineSession> {
        self.sessions.iter()
    }

    pub fn sessions_mut(&mut self) -> impl Iterator<Item=&mut TimelineSession> {
        self.sessions.iter_mut()
    }
    
    /// Plays the animation with the given name and returns its state before playing.
    /// Returns `None` if the target animation does not exist
    pub fn play(&mut self, name:&str, option:Option<TimelinePlayOption>) -> Option<bool> {
        if let Some( session_idx) = self.shortcut.get( name ) {
            let session = &mut self.sessions[ *session_idx ];
            let is_playing = session.is_playing;
            session.play( option.unwrap_or_default() );
            Some(is_playing)
        } else {
            None
        }
    }

    pub fn play_id(&mut self, name:Timeline, option:TimelinePlayOption) {

    }

    /// Stops the animation with the given name and returns its state before playing. 
    /// Returns `None` if the target animation does not exist
    pub fn stop(&mut self, name:&str) -> Option<bool> {
        if let Some(session) = self.sessions.get_mut(name) {
            let is_playing = session.is_playing;
            session.stop();
            Some(is_playing)
        } else {
            None
        }
    }
    
    pub fn is_playing(&self, name:&str) -> bool {
        if let Some(session) = self.sessions.get(name) {
            session.is_playing
        } else {
            false
        }
    }

    pub fn is_something_playing(&self) -> bool {
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
    
    /// Replay any internal non-zero accumulated playback time.
    pub fn resume_all(&mut self) {
        
    }
}