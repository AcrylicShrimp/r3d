use crate::AssetDatabase;
use asset::TypedAsset;
use asset_pipeline::AssetProcessError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("asset not found: {0}")]
    AssetNotFound(Uuid),
    #[error("failed to process asset: {0}")]
    ProcessError(#[from] AssetProcessError),
    #[error("failed to load asset: {0}")]
    LoadError(#[from] asset::AssetLoadError),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

pub trait AssetLoader {
    fn load_asset(&self, id: Uuid, database: &AssetDatabase) -> Result<TypedAsset, AssetLoadError>;
}
