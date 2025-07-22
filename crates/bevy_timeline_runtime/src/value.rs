

use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::{ComponentMutability, Mutable};
use bevy_reflect::TypePath;
use bevy_ecs::prelude::{Component};
use bevy_ecs::query::QueryData;
use bevy_math::{Vec3, Quat};
use bevy_transform::prelude::{Transform};

use crate::{TimelineError};
use crate::data::{TimelineKeyframe, TimelineUntypedKeyframes};

#[cfg(feature="json")]
mod json {
    use serde_json::{json,Value};
}

#[cfg(not(feature="json"))]
mod ron {
    use std::ops::Deref;
    use bevy_asset::ron::Value;
    use crate::TimelineError;

    pub struct Var(Value);

    impl TryFrom<&Var> for bool {
        type Error = TimelineError;

        fn try_from(value: &Var) -> Result<Self, Self::Error> {

            match value.0 {
                Value::Bool(v) => Ok(v),
                _ => Err(TimelineError::IncorrectValueType("not bool"))
            }
        }
    }

    impl TryFrom<&Var> for char {
        type Error = TimelineError;

        fn try_from(value: &Var) -> Result<Self, Self::Error> {
            match value.0 {
                Value::Char(v) => Ok(v),
                _ => Err(TimelineError::IncorrectValueType("not char"))
            }
        }
    }

    impl TryFrom<&Var> for String {
        type Error = TimelineError;

        fn try_from(value: &Var) -> Result<Self, Self::Error> {
            match value.0 {
                Value::String(ref v) => Ok(v.clone()),
                _ => Err(TimelineError::IncorrectValueType("not string"))
            }
        }
    }


    impl Var {
        pub fn as_to<T:for<'a> TryFrom<&'a Var,Error=TimelineError>>(&self) -> Result<T, TimelineError> {
            T::try_from(self)
        }

        pub fn get<T:for<'a> TryFrom<&'a Var,Error=TimelineError>>(&self, key:&'static str) -> Result<T, TimelineError> {
            match self.0 {
                Value::Map(ref m) => {
                    for (k,v) in m.iter() {
                        if let Value::String( s) = k {
                            if s == key {
                                return T::try_from( &Var(v.clone()) )
                            }
                        }
                    }
                    Err(TimelineError::ItemNotExist(key))
                }
                _ => Err(TimelineError::IncorrectValueType("not map"))
            }
        }

        pub fn get_array<K:AsRef<str>,T:for<'a> TryFrom<&'a Var,Error=TimelineError>>(&self, key:K) -> Result<Vec<T>, TimelineError> {
            match self.0 {
                Value::Seq(ref v) => {
                    let mut list = Vec::with_capacity(v.len());
                    for v in v {
                        list.push( T::try_from( &Var(v.clone()) )? );
                    }
                    Ok(list)
                },
                _ => Err(TimelineError::IncorrectValueType("not array"))
            }
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

    fn from_value(value:&Var) -> Result<Self, TimelineError>;

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
        std::any::type_name::<Self>()
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
