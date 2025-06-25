use std::borrow::Cow;
use std::marker::PhantomData;
use std::string::ToString;
use bevy_reflect::TypePath;
use bevy_asset::{Asset};
use bevy_transform::prelude::Transform;
use crate::value::{AnimatableValue};

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
    
    pub fn load_frame(value:&Value) -> Result<Keyframe<V>, Cow<'static,str>> {
        let keyframe = value.as_object().ok_or( Cow::Borrowed("keyframe is not object") )?;
        let time = keyframe.get("time").ok_or( Cow::Borrowed("time(in keyframe) is not exist") )?.as_f64().ok_or( Cow::Borrowed("time(in keyframe) is not number") )? as f32;
        let data = V::from_value(None, keyframe.get("data").ok_or( Cow::Borrowed("data(in keyframe) is not exist") )? )?;
        Ok( Keyframe::new( time,data ) )
    }

    pub fn load_frames(values:&Vec<Value>) -> Result<Vec<Keyframe<V>>, Cow<'static,str>> {
        let mut keyframes = Vec::<Keyframe<V>>::new();
        for keyframe in values {
            keyframes.push( Keyframe::load_frame(keyframe)? );
        }
        Ok(keyframes)
    }
}


#[derive(TypePath,Asset)]
pub struct Timeline<K> where K:AnimatableValue+Send+Sync+TypePath {
    name: String,
    duration: f32,
    keyframes : Vec<Keyframe<K>>
}

impl <K> Timeline<K> where K:AnimatableValue+Send+Sync {
    pub fn new(name:String, duration:f32, keyframes:Vec<Keyframe<K>>) -> Timeline<K> {
        Self { name, duration, keyframes }
    }

    pub fn find_keyframe_pair(&self, time:f32) -> (Option<&Keyframe<K>>,Option<&Keyframe<K>>) {
        if self.keyframes.is_empty() {
            return (None, None);
        }

        // if time < self.frames[0].time {
        //     return (None, Some( &self.frames[0] ));
        // }
        // 
        // if time >= self.frames[self.frames.len() - 1].time {
        //     return (Some( &self.frames[self.frames.len() - 1] ), None);
        // }

        let (before,next) = match self.keyframes.binary_search_by(|probe| probe.time.partial_cmp(&time).unwrap()) {
            Ok(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = Some(i);
                (before, next)
            }
            Err(i) => {
                let before = if i > 0 { Some(i - 1) } else { None };
                let next = if i < self.keyframes.len() { Some(i) } else { None };
                (before, next)
            }
        };
        (before.map(|idx| &self.keyframes[idx]), next.map(|idx| &self.keyframes[idx]))
    }

    pub fn interpolate(&self, anim_time:f32, out:&mut K::Target) {
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

    // fn from_value(value:&Value) -> Result<Timeline<K>, Cow<'static,str>> {
    //     let timeline = value.as_object().ok_or(Cow::Borrowed("timeline is not object"))?;
    //     let name = timeline.get("name").ok_or(Cow::Borrowed("name(in timeline) is not exist"))?.as_str().ok_or(Cow::Borrowed("name(in timeline) is not string"))?.to_string();
    //     let duration = timeline.get("duration").ok_or(Cow::Borrowed("duration(in timeline) is not exist)"))?
    //         .as_number().ok_or(Cow::Borrowed("duration(in timeline) is not number"))?.as_f64().unwrap() as f32;
    //     let keyframes = timeline.get("keyframes").ok_or(Cow::Borrowed("keyframes(in timeline) is not exist"))?;
    //     let keyframes = Keyframe::<K>::load_frames(keyframes)?;
    //     Ok(Self { name, duration, keyframes })
    // }
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