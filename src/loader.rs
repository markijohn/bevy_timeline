use bevy_asset::io::Reader;
use bevy_asset::{AssetLoader, LoadContext};
use thiserror::Error;
use crate::TimelineRawData;
use std::str::FromStr;
use bevy_asset::AsyncReadExt;

pub struct TimelineRawDataLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonLoaderError {
    /// [IO Error](std::io::Error)
    #[error("JSON data load failed: {0}")]
    Io(#[from] std::io::Error),
    /// [JSON Error](serde_json::error::Error)
    #[error("Could not parse the JSON: {0}")]
    JsonError(#[from] serde_json::error::Error),
}

impl AssetLoader for TimelineRawDataLoader {
    type Asset = TimelineRawData;
    type Settings = ();
    type Error = JsonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
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