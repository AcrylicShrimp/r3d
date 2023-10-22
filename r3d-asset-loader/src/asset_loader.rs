use crate::AssetDatabase;
use asset::{AssetKey, TypedAsset};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("asset not found: {0}")]
    AssetNotFound(Uuid),
    #[error("failed to deduce asset type: {0}")]
    AssetTypeDeduceError(#[from] asset_pipeline::AssetTypeDeduceError),
    #[error("failed to process asset: {0}")]
    ProcessError(#[from] asset_pipeline::AssetProcessError),
    #[error("failed to load asset: {0}")]
    LoadError(#[from] asset::AssetLoadError),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

pub trait AssetLoader {
    fn load_asset(
        &self,
        key: &AssetKey,
        database: &AssetDatabase,
    ) -> Result<TypedAsset, AssetLoadError>;
}
