use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use bevy_asset::io::Reader;
use bevy_asset::{AssetLoader, LoadContext};
use thiserror::Error;
use std::str::FromStr;
use bevy_asset::AsyncReadExt;
use crate::data::{TimelineAnimation, TimelineAnimationSet};
use crate::{TimelineError, TimelineImplSets};

pub struct TimelineAnimationSetLoader<K> where K:TimelineImplSets {
    inner: PhantomData<K>
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
                load_context.add_labeled_asset(anim.name.clone(), anim)
            );
        });
        Ok( TimelineAnimationSet(anim_sets) )
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}