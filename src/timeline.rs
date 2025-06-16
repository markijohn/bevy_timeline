use std::marker::PhantomData;
use bevy_reflect::TypePath;
use bevy_asset::{Asset};
use bevy_transform::prelude::Transform;
use crate::value::{AnimatableValue, TLTransform};

pub struct Keyframe<V:AnimatableValue<Out>, Out> {
    pub time:f32,
    pub value:V,
    inner : PhantomData<Out>
}

impl <V:AnimatableValue<Out>,Out> Keyframe<V,Out> {
    pub fn new(time:f32,value:V) -> Keyframe<V, Out> {
        Self { time, value, inner : PhantomData }
    }

    pub fn lerp(&self, elapsed:f32, next:&Keyframe<V,Out>, out:&mut V) {
        self.value.interpolate(elapsed, &next.value, out);
    }
}

#[derive(TypePath,Asset)]
pub struct Timeline<K=TLTransform,Out=Transform> {
    playtime : f32,
    frames : Vec<Keyframe<K,Out>>
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

    pub fn lerp(&self, anim_time:f32, out:&mut K) {
        let (bef,next) = self.find_keyframe_pair(anim_time);
        match (bef, next) {
            (Some(bef), None) => {
                //end of keyframe
            }
            (None, Some(next)) => {
                //no start keyframe
                todo!()
            }
            (Some(bef), Some(next)) => {
                let time_diff = next.time - bef.time;
                let elapsed = (anim_time - bef.time) / time_diff;
                bef.lerp(elapsed, next, out);
            }
            (None, None) => {
                //No frames
            }
        }
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