use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::{Mutable};
use bevy_ecs::prelude::{Component};
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{TimelineError};
use crate::data::{TimelineUntypedSeq};


pub trait ValueExt {
    fn as_f32(&self) -> Result<f32, TimelineError>;
    fn as_fixed_array_f32<const SIZE:usize>(&self) -> Result<[f32;SIZE], TimelineError>;
    fn as_array_f32(&self) -> Result<Vec<f32>, TimelineError>;

    fn child(&self, key:&'static str) -> Option<&Self>;

    fn get_f32(&self, key:&'static str) -> Result<f32, TimelineError> {
        if let Some(child) = self.child(key) {
            child.as_f32()
        } else {
            Err(TimelineError::NotExistItem(key))
        }
    }

    fn get_fixed_array_f32<const SIZE:usize>(&self, key:&'static str) -> Result<[f32;SIZE], TimelineError> {
        if let Some(v) = self.child(key) {
            v.as_fixed_array_f32::<SIZE>()
        } else {
            Err(TimelineError::NotExistItem(key))
        }
    }

    fn get_array_f32(&self, key:&'static str) -> Result<Vec<f32>, TimelineError> {
        if let Some(v) = self.child(key) {
            v.as_array_f32()
        } else {
            Err(TimelineError::NotExistItem(key))
        }
    }
}

impl ValueExt for Value {
    fn as_f32(&self) -> Result<f32, TimelineError> {
        Ok(
            self.as_f64().ok_or( TimelineError::IncorrectValueType("not a number") )? as f32
        )
    }

    fn as_fixed_array_f32<const SIZE: usize>(&self) -> Result<[f32; SIZE], TimelineError> {

        if let Value::Array(vec) = self {
            if vec.len() != SIZE {
                return Err(TimelineError::InvalidLength {required:SIZE, actual: vec.len()});
            }
            let mut v = [0f32;SIZE];
            for i in 0..SIZE {
                v[i] = vec[i].as_f32()?;
            }
            Ok(v)
        } else {
            Err(TimelineError::IncorrectValueType("not array"))
        }
    }

    fn as_array_f32(&self) -> Result<Vec<f32>, TimelineError> {
        if let Value::Array(vec) = self {
            let mut v = Vec::with_capacity(vec.len());
            for i in vec {
                v.push(i.as_f32()?);
            }
            Ok(v)
        } else {
            Err(TimelineError::IncorrectValueType("not array"))
        }
    }

    fn child(&self, key: &'static str) -> Option<&Self> {
        if let Value::Object(map) = self {
            map.get(key)
        } else {
            None
        }
    }

}

pub trait AnimatableValue:Default+Clone+Sized+Serialize+for<'a> Deserialize<'a>+'static {
    type Target: Component<Mutability=Mutable>;
    
    /// TODO : fast keyframe search from cached(last searched index, last proceed time)
    /// `play_time` : progress play time
    /// `pre_idx` : proceed keyframe index
    /// `keyframes` : all keyframes
    /// `out` : output
    /// return : Some(usize) : changed key frame index, None : keyframe not changed
    fn interpolate_from_keyframe(_duration:f32, _prev_time:f32, curr_time:f32, times:&[f32], seq:&[Self], mut out:Mut<Self::Target>) {
        if times.is_empty() {
            return;
        }

        let (before,next) = match times.binary_search_by(|time| time.partial_cmp(&curr_time).unwrap()) {
            Ok(i) => {
                let before = if i > 0 {
                    Some(i - 1)
                } else { None };
                let next = Some(i);
                (before, next)
            }
            Err(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = if i < times.len() { Some(i) } else { None };
                (before, next)
            }
        };
        let (start,end) = (
            before.map(|idx| (times[idx], seq[idx].clone()) ),
            next.map(|idx| (times[idx], seq[idx].clone()) )
        );

        match (start, end) {
            (Some( (start,data) ), None) => {
                // TODO : If there is no next keyframe to process and the previous keyframe processed is 
                // the same as the start keyframe, no processing is required, i.e., no Mut value is substituted, which prevents bevy from being marked Changed.
                //end of keyframe
                Self::interpolate(0., data, None, out.as_mut());
            }
            (Some( (start,start_data) ), Some( (end, end_data) ) ) => {
                let time_diff = end - start;
                let s = (curr_time - start) / time_diff;
                Self::interpolate(s, start_data, Some(end_data), out.as_mut());
            }
            _ => {
                //No frames
            }
        }
    }
    
    fn interpolate(s:f32, start:Self, end:Option<Self>, out:&mut Self::Target);

    fn create_untyped_keyframes(value:&[Value]) -> Result<TimelineUntypedSeq, TimelineError> {
        let mut keyframes = Vec::<Self>::with_capacity( value.len() );
        for i in value {
            keyframes.push( serde_json::from_value( i.clone() )? );
        }
        let addr = keyframes.as_mut_ptr() as usize;
        let length = keyframes.len();
        let capacity = keyframes.capacity();
        keyframes.leak();
        let dropper = Box::new( move || {
            unsafe { Vec::from_raw_parts(addr as *mut Self, length, capacity); }
        });
        Ok( TimelineUntypedSeq {
            typ: Self::typ(),
            addr,
            length,
            dropper
        })
    }

    fn typ() -> &'static str {
        //std::any::type_name::<Self>()
        let full_name = std::any::type_name::<Self>();
        let name = if let Some(pos) = full_name.rfind("::") {
            &full_name[pos + 2..]
        } else {
            full_name
        };

        // if let Some(pos) = name.find('<') {
        //     &name[..pos]
        // } else {
        //     &name
        // }
        name
    }
}

#[derive(Serialize,Deserialize,Default,Clone)]
pub struct Scale(Vec3);

impl AnimatableValue for Scale {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.scale = start.0 + (end.0 - start.0) * s;
        } else {
            out.scale = start.0;
        }
    }
}

#[derive(Serialize,Deserialize,Default,Clone)]
pub struct Rotation(Quat);

impl AnimatableValue for Rotation {
    type Target = Transform;

    fn interpolate(s: f32, prev:Self, next: Option<Self>, out: &mut Self::Target) {
        if let Some(next) = next {
            out.rotation = prev.0.slerp( next.0, s );
        } else {
            out.rotation = prev.0;
        }
    }
}


#[derive(Serialize,Deserialize,Default,Clone)]
pub struct Translation(Vec3);

impl AnimatableValue for Translation {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.translation = start.0 + (end.0 - start.0) * s;
        } else {
            out.translation = start.0;
        }
    }
}
