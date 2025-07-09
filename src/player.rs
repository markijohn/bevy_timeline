use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use crate::AnimatableValue;
use crate::data::{TimelineAnimation};


#[derive(Clone,Default)]
pub enum TimelinePlayback {
    #[default]
    Forward,        // Play forward from current position until animation duration
    ForwardLoop,    // Play forward continuously until stopped
    Backward,       // Play backward from current position until animation duration
    BackwardLoop,   // Play backward continuously until stopped
    StepByRate(f32), // Advance by specified rate (next state will be Stop)
    StepByTime(f32), // Advance by specified time (next state will be Stop)
    Pause(Box<TimelinePlayback>),          // Pause playback at current position (can be resumed)
    Stop            // Stop playback and reset to initial state
}

#[derive(Default)]
pub enum TimelineDuration {
    /// Play with the original animation duration
    #[default]
    Original,

    /// Play within the specified time duration. If shorter than animation, 
    /// animation will be cut off. If longer, animation will be extended at the end.
    CropOrExtendAtEnd(f32),

    /// Play within the specified time duration. If shorter than animation,
    /// the difference is added to the start delay. If longer, animation is extended at the beginning.
    CropOrExtendAtStart(f32),

    /// Play with time scaling. 1.0 = normal speed, 0.5 = double speed, 2.0 = half speed
    TimeScale(f32),
}

pub enum TimelinePosition {
    /// Relative position (0.0 = start, 1.0 = end)
    Normalized(f32), // position = duration * normalized_value

    /// Absolute time position in seconds
    Absolute(f32), // position = absolute_time
}

impl Default for TimelinePosition {
    fn default() -> Self {
        Self::Absolute(0.0)
    }
}

pub struct TimelineProgress {
    pub duration:f32,
    pub prev_time:f32,
    pub curr_time:f32
}

pub struct TimelineTargetBinded {
    pub typ:&'static str,
    pub entity: Entity,
    pub target_idx: usize
}

pub struct TimelineSession {
    pub is_loop_interpolation : bool,
    pub duration : f32,
    pub prev_progress: f32,
    pub progress : f32,
    pub playback : TimelinePlayback,
    pub play_duration: TimelineDuration,
    pub anim_handle: Handle<TimelineAnimation>,
    binded_targets: Vec<TimelineTargetBinded>,
}

impl TimelineSession {
    /// binded child (entity,timeline)
    pub fn binded_targets(&self) -> &[TimelineTargetBinded] {
        self.binded_targets.as_slice()
    }

    /// The `play` function always starts at time `0` (but can depend on the start of `TimelinePlayOption`)
    pub fn play(&mut self, option:TimelinePlayOption) {
        self.progress = 0.;
        if let Some(start) = option.position {
            match start {
                TimelinePosition::Normalized(r) => { self.progress = self.duration * r }
                TimelinePosition::Absolute(t) => { self.progress = t }
            }
        }
        self.playback = option.playback.unwrap_or_default()
    }

    /// Resume from the point where `stop` was called
    pub fn resume(&mut self) {
        if let TimelinePlayback::Pause(old_playback) = &self.playback {
            self.playback = (*(old_playback.clone())).into();
        }
    }

    /// Pauses the animation playing in the current session
    pub fn stop(&mut self) {
        self.playback = TimelinePlayback::Stop;
    }
    
    pub fn pause(&mut self) {
        if let TimelinePlayback::Pause(_) = &self.playback {
            return;
        }
        self.playback = TimelinePlayback::Pause(Box::new(self.playback.clone()));
    }

    pub fn is_playing(&self) -> bool {
        match self.playback {
            TimelinePlayback::Pause(_) | TimelinePlayback::Stop => false,
            _ => false
        }
    }

    pub fn status(&self) -> TimelinePlayback {
        self.playback.clone()
    }
    
    /// Accumulate playback time
    pub fn update_time(&mut self, elapsed:f32) {
        self.prev_progress = self.progress;
        match self.playback {
            TimelinePlayback::Forward => { self.progress += elapsed; self.progress = self.progress.min(self.duration) },
            TimelinePlayback::ForwardLoop => self.progress = (self.progress + elapsed) % self.duration,
            TimelinePlayback::Backward => { self.progress -= elapsed; self.progress = self.progress.max(self.duration) },
            TimelinePlayback::BackwardLoop => self.progress = (self.progress - elapsed).abs() % self.duration,
            TimelinePlayback::StepByRate(r) => self.progress = self.duration * r,
            TimelinePlayback::StepByTime(t) => self.progress = t,
            TimelinePlayback::Pause(_) => (),
            TimelinePlayback::Stop => (),
        }
    }
}

#[derive(Default)]
pub struct TimelinePlayOption {
    position: Option<TimelinePosition>,
    playback: Option<TimelinePlayback>,
    duration: Option<TimelineDuration>,
}

impl TimelinePlayOption {
    /// Set the playback start time
    pub fn set_position(mut self, start:TimelinePosition) -> Self{
        self.position = Some(start);
        self
    }
    
    /// Set loop mode
    pub fn set_playback(mut self, playback:TimelinePlayback) -> Self{
        self.playback = Some(playback);
        self
    }
    
    /// Set play mode
    pub fn set_duration(mut self, duration:TimelineDuration) -> Self{
        self.duration = Some(duration);
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

    pub fn create_session(&mut self, anim_handle:Handle<TimelineAnimation>, binded_targets:Vec<TimelineTargetBinded>) {
        self.sessions.push(TimelineSession {
            is_loop_interpolation: true,
            duration: 0.0,
            prev_progress: 0.0,
            progress: 0.0,
            playback: TimelinePlayback::Stop,
            play_duration: TimelineDuration::Original,
            anim_handle,
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
    pub fn play(&mut self, name:&str, option:Option<TimelinePlayOption>) {
        if let Some( session_idx) = self.shortcut.get( name ) {
            let session = &mut self.sessions[ *session_idx ];
            session.play( option.unwrap_or_default() );
        }
    }

    /// Stops the animation with the given name and returns its state before playing.
    pub fn stop(&mut self, name:&str) {
        if let Some(idx) = self.shortcut.get_mut(name) {
            let session = &mut self.sessions[*idx];
            session.stop();
        }
    }

    pub fn playing_state(&self) -> impl Iterator<Item=(&str, TimelinePlayback)> {
        self.shortcut.iter().map(|(name,idx)| (name.as_str(), self.sessions[*idx].playback.clone()))
    }

    pub fn stop_all(&mut self) {
        self.sessions.iter_mut().for_each( |e| {
            e.stop();
        })
    }
    
    pub fn pause(&mut self, name:&str) {
        if let Some(idx) = self.shortcut.get_mut(name) {
            let session = &mut self.sessions[*idx];
            session.stop();
        }
    }

    pub fn resume(&mut self, name:&str) {
        if let Some(idx) = self.shortcut.get_mut(name) {
            self.sessions[*idx].resume();
        }
    }

    pub fn resume_all(&mut self) {
        self.sessions.iter_mut().for_each( |e| e.resume() );
    }

    pub fn get_playing_entities<T:AnimatableValue>( &self ) -> impl Iterator<Item=&(TimelineProgress,&TimelineTargetBinded)> {
        self.sessions.iter()
            .filter(|v| v.is_playing())
            .map(|v| {
                let prg = TimelineProgress {
                    duration: v.duration,
                    prev_time: v.prev_progress,
                    curr_time: v.progress,
                };
                v.binded_targets.iter().map( |v| (prg,v) )
            } )
            .flatten()
            .filter( |v| v.1.typ == T::typ() )
    }
}