use std::borrow::Cow;
use std::collections::HashMap;
use bevy_asset::io::Reader;
use bevy_asset::{AssetLoader, LoadContext};
use thiserror::Error;
use std::str::FromStr;
use bevy_asset::AsyncReadExt;
use crate::data::{TimelineAnimationSet, TimelineUntypedAnimation, TimelineUntypedResolver};



#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonLoaderError {
    /// [IO Error](std::io::Error)
    #[error("JSON data load failed: {0}")]
    Io(#[from] std::io::Error),

    /// [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),

    UnknownError( Cow<'static,str> )
}

pub struct TimelineAnimationSetLoader(HashMap<&'static str,TimelineUntypedResolver>);

impl AssetLoader for TimelineAnimationSetLoader {
    type Asset = TimelineAnimationSet;
    type Settings = ();
    type Error = JsonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string).await?;
        let value = serde_json::Value::from_str(&string)?;
        Ok( crate::data::TimelineAnimationSet::from(&self.0, value)? )
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

pub struct TimelineAnimationLoader(HashMap<&'static str,TimelineUntypedResolver>);

impl AssetLoader for TimelineAnimationSetLoader {
    type Asset = TimelineUntypedAnimation;
    type Settings = ();
    type Error = JsonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut string = String::new();
        reader.read_to_string(&mut string).await?;
        let value = serde_json::Value::from_str(&string)?;
        Ok( TimelineRawData(value) )
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
// use bevy_asset::io::Reader;
// use bevy_asset::{AssetLoader, LoadContext};
// use thiserror::Error;
// use crate::TimelineRawData;
// use std::str::FromStr;
// use bevy_asset::AsyncReadExt;
//
// pub struct TimelineRawDataLoader;
//
// #[non_exhaustive]
// #[derive(Debug, Error)]
// pub enum JsonLoaderError {
//     /// [IO Error](std::io::Error)
//     #[error("JSON data load failed: {0}")]
//     Io(#[from] std::io::Error),
//     /// [JSON Error](serde_json::error::Error)
//     #[error("Could not parse the JSON: {0}")]
//     JsonError(#[from] serde_json::error::Error),
// }
//
// impl AssetLoader for TimelineRawDataLoader {
//     type Asset = TimelineRawData;
//     type Settings = ();
//     type Error = JsonLoaderError;
//
//     async fn load(
//         &self,
//         reader: &mut dyn Reader,
//         _settings: &(),
//         load_context: &mut LoadContext<'_>,
//     ) -> Result<Self::Asset, Self::Error> {
//         load_context.
//         let mut string = String::new();
//         reader.read_to_string(&mut string).await?;
//         let value = serde_json::Value::from_str(&string)?;
//         Ok( TimelineRawData(value) )
//     }
//
//     fn extensions(&self) -> &[&str] {
//         &["json"]
//     }
// }