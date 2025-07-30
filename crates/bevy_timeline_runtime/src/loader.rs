use std::marker::PhantomData;
use bevy_asset::io::Reader;
use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use std::str::FromStr;
use bevy_asset::AsyncReadExt;
use crate::data::{TimelineAnimation, TimelineAnimationSet};
use crate::{TimelineError, TimelineImplSets};

#[derive(Default)]
pub struct TimelineAnimationSetLoader<K> {
    inner: PhantomData<K>
}

impl<K> TimelineAnimationSetLoader<K> {
    pub fn new() -> Self {
        Self {
            inner: PhantomData
        }
    }
}

impl <K> AssetLoader for TimelineAnimationSetLoader<K> where K:TimelineImplSets + Send + Sync + 'static {
    type Asset = TimelineAnimationSet;
    type Settings = ();
    type Error = TimelineError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {

        let mut string = String::new();
        reader.read_to_string(&mut string).await?;
        let value = serde_json::Value::from_str(&string)?;
        let anims = TimelineAnimation::load_animations::<K>( &value )?;
        let mut anim_sets = Vec::with_capacity(anims.len());
        
        anims.into_iter().for_each( |anim| {
            anim_sets.push(
                load_context.add_loaded_labeled_asset(anim.name.clone(), LoadedAsset::from(anim))
            );
        });
        Ok( TimelineAnimationSet {
            path: Some( load_context.path().to_path_buf() ),
            anim_handles : anim_sets
        } )
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}