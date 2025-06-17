use std::borrow::Cow;
use std::marker::PhantomData;
use std::string::ToString;
use bevy_reflect::TypePath;
use bevy_asset::{Asset};
use bevy_reflect::erased_serde::__private::serde::Deserializer;
use bevy_transform::prelude::Transform;
use crate::value::{AnimatableValue, TLTransform};

#[cfg(feature = "json_serialize")]
use serde_json::Value;

pub struct Keyframe<V:AnimatableValue> {
    pub time:f32,
    pub value:V,
}

impl <V:AnimatableValue> Keyframe<V> {
    pub fn new(time:f32,value:V) -> Keyframe<V> {
        Self { time, value }
    }

    pub fn interpolate(&self, s:f32, next:&Keyframe<V>, out:&mut V::Target) {
        self.value.interpolate(s, &next.value, out);
    }
}

#[cfg(feature="json_serialize")]
mod group {
    use serde_json::Value;

    pub struct TimelineTrack {
        pub typ: String,
        pub name: String,
        pub raw_keyframes: Vec<Value>,
    }
    pub struct TimelineGroupBulk {
        duration : f32,
        track : Vec<TimelineTrack>
    }
}

#[derive(TypePath,Asset)]
pub struct Timeline<K=TLTransform> {
    frames : Vec<Keyframe<K>>
}

impl <K> Default for Timeline<K> where K:AnimatableValue + Asset {
    fn default() -> Self {
        Self { frames: vec![] }
    }
}

impl <K> Timeline<K> where K:AnimatableValue + Asset {
    pub fn new(frames:Vec<Keyframe<K>>) -> Timeline<K> {
        Self { frames }
    }

    pub fn find_keyframe_pair(&self, time:f32) -> (Option<&Keyframe<K>>,Option<&Keyframe<K>>) {
        if self.frames.is_empty() {
            return (None, None);
        }

        // if time < self.frames[0].time {
        //     return (None, Some( &self.frames[0] ));
        // }
        // 
        // if time >= self.frames[self.frames.len() - 1].time {
        //     return (Some( &self.frames[self.frames.len() - 1] ), None);
        // }

        let (before,next) = match self.frames.binary_search_by(|probe| probe.time.partial_cmp(&time).unwrap()) {
            Ok(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = Some(i);
                (before, next)
            }
            Err(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = if i < self.frames.len() { Some(i) } else { None };
                (before, next)
            }
        };
        (before.map(|idx| &self.frames[idx]), next.map(|idx| &self.frames[idx]))
    }

    pub fn lerp(&self, anim_time:f32, out:&mut K::Target) {
        let (bef,next) = self.find_keyframe_pair(anim_time);
        match (bef, next) {
            (Some(bef), None) => {
                //end of keyframe
            }
            (None, Some(next)) => {
                //no start keyframe
            }
            (Some(bef), Some(next)) => {
                let time_diff = next.time - bef.time;
                let s = (anim_time - bef.time) / time_diff;
                bef.interpolate(s, next, out);
            }
            (None, None) => {
                //No frames
            }
        }
    }

    #[cfg(feature="json_serialize")]
    fn from_value(value:&Value) -> Result<Self<K>, Cow<'static,str>> {
        const KEY_TIME:Value = Value::String("time".to_string());
        let keyframes = value.as_array().ok_or( Cow::Borrowed("value is not array('keyframes')") )?;
        let mut frames = Vec::with_capacity(keyframes.len());
        for key in keyframes {
            let map = key.as_object().ok_or( Cow::Borrowed("value is not object(`keyframe`)") )?;
            let time = map.get("time").ok_or( Cow::Borrowed("time not exist in keyframe") )?
                .as_number().ok_or( Cow::Borrowed("time is not a number") )?
                .as_f64().unwrap() as f32;
            let data = K::from_value( map.get("data").ok_or( Cow::Borrowed("data not exist in keyframe") )? )?;
            frames.push( Keyframe::new(time, data) );
        }
        Ok( Self { frames } )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn binary_search() {
        struct Frames {
            frames: Vec<f32>,
        }

        impl Frames {
            fn find(&self, time: f32) -> Option<(f32, f32)> {
                let frames = &self.frames;

                if frames.len() < 2 {
                    return None;
                }

                match frames.binary_search_by(|probe| probe.partial_cmp(&time).unwrap()) {
                    Ok(i) => {
                        // find exact
                        if i + 1 < frames.len() && i > 0 {
                            Some((frames[i], frames[i + 1]))
                        } else {
                            None
                        }
                    }
                    Err(i) => {
                        if i == 0 || i >= frames.len() {
                            // 범위를 벗어남
                            None
                        } else {
                            Some((frames[i - 1], frames[i]))
                        }
                    }
                }
            }
        }

        let frames = Frames {
            frames: vec![3.0, 6.0, 7.0, 9.0],
        };

        let result = frames.find(5.5);
        println!("{:?}", result); // Some((3.0, 6.0))

        let result = frames.find(7.0);
        println!("{:?}", result); // Some((7.0, 9.0))

        let result = frames.find(2.0);
        println!("{:?}", result); // None

        let result = frames.find(10.0);
        println!("{:?}", result); // None
    }
}