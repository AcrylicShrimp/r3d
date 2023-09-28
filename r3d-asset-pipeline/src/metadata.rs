use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum MetadataLoadError {
    #[error("toml error: {0}")]
    TOMLError(#[from] toml::de::Error),
}

/// Metadata for an asset.
#[derive(Serialize, Deserialize)]
pub struct Metadata<T> {
    pub asset: AssetMetadata,
    #[serde(flatten)]
    pub extra: T,
}

/// Standard asset metadata under `asset` table.
#[derive(Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: Uuid,
}

impl<T> Metadata<T>
where
    T: for<'de> Deserialize<'de>,
{
    pub fn from_toml(content: impl AsRef<str>) -> Result<Self, MetadataLoadError> {
        toml::from_str(content.as_ref()).map_err(MetadataLoadError::from)
    }
}
