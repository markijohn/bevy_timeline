use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::{Mutable};
use bevy_ecs::prelude::{Component};
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};
use serde_json::Value;
use crate::{TimelineError};
use crate::data::{TimelineKeyframe, TimelineUntypedKeyframes};


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

pub trait AnimatableValue:Default+Clone+Sized+'static {
    type Target: Component<Mutability=Mutable>;
    
    /// TODO : fast keyframe search from cached(last searched index, last proceed time)
    /// `play_time` : progress play time
    /// `pre_idx` : proceed keyframe index
    /// `keyframes` : all keyframes
    /// `out` : output
    /// return : Some(usize) : changed key frame index, None : keyframe not changed
    fn interpolate_from_keyframe(_duration:f32, _prev_time:f32, curr_time:f32, keyframes:&[TimelineKeyframe<Self>], mut out:Mut<Self::Target>) {
        if keyframes.is_empty() {
            return;
        }

        let (before,next) = match keyframes.binary_search_by(|probe| probe.time.partial_cmp(&curr_time).unwrap()) {
            Ok(i) => {
                let before = if i > 0 {
                    Some(i - 1)
                } else { None };
                let next = Some(i);
                (before, next)
            }
            Err(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = if i < keyframes.len() { Some(i) } else { None };
                (before, next)
            }
        };
        let (start,end) = (
            before.map(|idx| keyframes[idx].clone()),
            next.map(|idx| keyframes[idx].clone())
        );

        match (start, end) {
            (Some(start), None) => {
                // TODO : If there is no next keyframe to process and the previous keyframe processed is 
                // the same as the start keyframe, no processing is required, i.e., no Mut value is substituted, which prevents bevy from being marked Changed.
                //end of keyframe
                Self::interpolate(0., start.data, None, out.as_mut());
            }
            (Some(start), Some(end)) => {
                let time_diff = end.time - start.time;
                let s = (curr_time - start.time) / time_diff;
                Self::interpolate(s, start.data, Some(end.data), out.as_mut());
            }
            _ => {
                //No frames
            }
        }
    }
    
    fn interpolate(s:f32, start:Self, end:Option<Self>, out:&mut Self::Target);

    fn from_value(value:&Value) -> Result<Self, TimelineError>;

    fn to_value(&self) -> Value;

    fn create_untyped_keyframes(value:&[Value]) -> Result<TimelineUntypedKeyframes, TimelineError> {
        // let keyframe = value.as_object().ok_or( TimelineError::IncorrectValueType("keyframe is not object") )?;
        // let time = keyframe.get("time").ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not exist") )?.as_f64().ok_or( TimelineError::IncorrectValueType("time(in keyframe) is not number") )? as f32;
        // let data = Self::from_value( keyframe.get("data").ok_or( TimelineError::IncorrectValueType("data(in keyframe) is not exist") )? )?;

        let mut keyframes = Vec::<TimelineKeyframe<Self>>::with_capacity( value.len() );
        for i in value {
            keyframes.push( TimelineKeyframe::<Self>::from(i)? );
        }
        let addr = keyframes.as_mut_ptr() as usize;
        let length = keyframes.len();
        let capacity = keyframes.capacity();
        keyframes.leak();
        let dropper = Box::new( move || {
            unsafe { Vec::from_raw_parts(addr as *mut Self, length, capacity); }
        });
        Ok( TimelineUntypedKeyframes {
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

        if let Some(pos) = name.find('<') {
            &name[..pos]
        } else {
            &name
        }
    }
}

#[derive(Default,Clone)]
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

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        let values = value.as_array().ok_or( TimelineError::IncorrectValueType("`Scale` keyframe must be [f32;3] array") )?;
        if values.len() != 3 {
            return Err( TimelineError::IncorrectValueType("`Scale` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( TimelineError::IncorrectValueType("`Scale`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( TimelineError::IncorrectValueType("`Scale`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( TimelineError::IncorrectValueType("`Scale`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let scale = self.0;
        Value::Array(vec![Value::from(scale.x), Value::from(scale.y), Value::from(scale.z)])
    }

    fn typ() -> &'static str {
        "Scale"
    }
}

#[derive(Default,Clone)]
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

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        let values = value.as_array().ok_or( TimelineError::IncorrectValueType("`Rotation` keyframe must be [f32;3] array") )?;
        if values.len() != 4 {
            return Err( TimelineError::IncorrectValueType("`Rotation` keyframe must be [f32;4] array") );
        }

        Ok( Self(Quat::from_array([
            values[0].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[3].as_number().ok_or( TimelineError::IncorrectValueType("`Rotation`[3] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let rotation = self.0;
        Value::Array(vec![Value::from(rotation.x), Value::from(rotation.y), Value::from(rotation.z), Value::from(rotation.w) ])
    }

    fn typ() -> &'static str {
        "Rotation"
    }
}


#[derive(Default,Clone)]
pub struct Translation(Vec3);

impl AnimatableValue for Translation {
    type Target = Transform;

    fn interpolate(s: f32, start:Self, end: Option<Self>, out: &mut Self::Target) {
        if let Some(end) = end {
            out.translation = start.0 + (end.0 - start.0) * s;
            // println!("translation(0) {}", out.translation);
        } else {
            out.translation = start.0;
            // println!("translation(1) {}", out.translation);
        }
    }

    fn from_value(value: &Value) -> Result<Self, TimelineError> {
        let values = value.as_array().ok_or( TimelineError::IncorrectValueType("`Translation` keyframe must be [f32;3] array") )?;
        if values.len() != 3 {
            return Err( TimelineError::IncorrectValueType("`Translation` keyframe must be [f32;3] array") );
        }
        Ok( Self(Vec3::from_array([
            values[0].as_number().ok_or( TimelineError::IncorrectValueType("`Translation`[0] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[1].as_number().ok_or( TimelineError::IncorrectValueType("`Translation`[1] is not a number(float)") )?.as_f64().unwrap() as f32,
            values[2].as_number().ok_or( TimelineError::IncorrectValueType("`Translation`[2] is not a number(float)") )?.as_f64().unwrap() as f32,
        ])) )
    }

    fn to_value(&self) -> Value {
        let translation = self.0;
        Value::Array(vec![Value::from(translation.x), Value::from(translation.y), Value::from(translation.z)])
    }

    fn typ() -> &'static str {
        "Translation"
    }
}
