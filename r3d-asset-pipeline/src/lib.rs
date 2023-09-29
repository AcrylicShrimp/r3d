use asset::{
    assets::{FontSource, ModelSource, TextureSource},
    AssetType,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

mod metadata;
mod pipeline;
pub mod pipelines;

pub use metadata::*;
pub use pipeline::*;

pub enum TypedAssetSource {
    Font(FontSource),
    Model(ModelSource),
    Texture(TextureSource),
}

impl From<FontSource> for TypedAssetSource {
    fn from(value: FontSource) -> Self {
        Self::Font(value)
    }
}

impl From<ModelSource> for TypedAssetSource {
    fn from(value: ModelSource) -> Self {
        Self::Model(value)
    }
}

impl From<TextureSource> for TypedAssetSource {
    fn from(value: TextureSource) -> Self {
        Self::Texture(value)
    }
}

#[derive(Error, Debug)]
pub enum AssetProcessError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("failed to load metadata: {0}")]
    MetadataLoadError(#[from] MetadataLoadError),
    #[error("failed to process asset: {0}")]
    AssetPipelineError(#[from] anyhow::Error),
}

pub fn process_asset(
    path: impl AsRef<Path>,
    asset_type: AssetType,
    metadata_content: impl AsRef<str>,
) -> Result<TypedAssetSource, AssetProcessError> {
    let path = path.as_ref();
    match asset_type {
        AssetType::Font => {
            let metadata = Metadata::from_toml(metadata_content)?;
            let file_content = std::fs::read(path)?;
            let asset = FontSource::process(file_content, &metadata)?;
            Ok(asset.into())
        }
        AssetType::Model => {
            let metadata = Metadata::from_toml(metadata_content)?;
            let file_content = std::fs::read(path)?;
            let asset = ModelSource::process(file_content, &metadata)?;
            Ok(asset.into())
        }
        AssetType::Shader => {
            todo!()
        }
        AssetType::Sprite => {
            todo!()
        }
        AssetType::Texture => {
            let metadata = Metadata::from_toml(metadata_content)?;
            let file_content = std::fs::read(path)?;
            let asset = TextureSource::process(file_content, &metadata)?;
            Ok(asset.into())
        }
    }
}

#[derive(Error, Debug)]
pub enum AssetTypeDeduceError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("path has no extension: {0}")]
    NoExtension(PathBuf),
    #[error("unsupported extension: {0}")]
    UnsupportedExtension(PathBuf),
}

pub fn deduce_asset_type_from_path(
    path: impl AsRef<Path>,
) -> Result<AssetType, AssetTypeDeduceError> {
    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| AssetTypeDeduceError::NoExtension(path.to_path_buf()))?;

    match extension.to_lowercase().as_str() {
        "ttf" | "otf" => Ok(AssetType::Font),
        "png" | "jpg" | "jpeg" | "gif" | "tif" | "tiff" | "tga" | "bmp" | "webp" => {
            Ok(AssetType::Texture)
        }
        _ => Err(AssetTypeDeduceError::UnsupportedExtension(
            path.to_path_buf(),
        )),
    }
}
