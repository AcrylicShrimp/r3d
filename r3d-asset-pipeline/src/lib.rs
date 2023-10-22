use asset::{
    assets::{FontSource, MaterialSource, ModelSource, ShaderSource, TextureSource},
    AssetType,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

mod metadata;
mod pipeline;
mod pipeline_gfx_bridge;
pub mod pipelines;

pub use metadata::*;
pub use pipeline::*;
pub use pipeline_gfx_bridge::*;

pub enum TypedAssetSource {
    Font(FontSource),
    Material(MaterialSource),
    Model(ModelSource),
    Shader(ShaderSource),
    Texture(TextureSource),
}

impl From<FontSource> for TypedAssetSource {
    fn from(value: FontSource) -> Self {
        Self::Font(value)
    }
}

impl From<MaterialSource> for TypedAssetSource {
    fn from(value: MaterialSource) -> Self {
        Self::Material(value)
    }
}

impl From<ModelSource> for TypedAssetSource {
    fn from(value: ModelSource) -> Self {
        Self::Model(value)
    }
}

impl From<ShaderSource> for TypedAssetSource {
    fn from(value: ShaderSource) -> Self {
        Self::Shader(value)
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
    metadata_content: Option<impl AsRef<str>>,
    gfx_bridge: &dyn PipelineGfxBridge,
) -> Result<TypedAssetSource, AssetProcessError> {
    let path = path.as_ref();
    match asset_type {
        AssetType::Font => {
            let metadata = metadata_content
                .map(|content| Metadata::from_toml(content))
                .transpose()?;
            let metadata = metadata.map(|metadata| metadata.extra).unwrap_or_default();
            let file_content = std::fs::read(path)?;
            let asset = FontSource::process(path, file_content, &metadata, gfx_bridge)?;
            Ok(asset.into())
        }
        AssetType::Material => {
            let metadata = metadata_content
                .map(|content| Metadata::from_toml(content))
                .transpose()?;
            let metadata = metadata.map(|metadata| metadata.extra).unwrap_or_default();
            let file_content = std::fs::read(path)?;
            let asset = MaterialSource::process(path, file_content, &metadata, gfx_bridge)?;
            Ok(asset.into())
        }
        AssetType::Model => {
            let metadata = metadata_content
                .map(|content| Metadata::from_toml(content))
                .transpose()?;
            let metadata = metadata.map(|metadata| metadata.extra).unwrap_or_default();
            let file_content = std::fs::read(path)?;
            let asset = ModelSource::process(path, file_content, &metadata, gfx_bridge)?;
            Ok(asset.into())
        }
        AssetType::Shader => {
            let metadata = metadata_content
                .map(|content| Metadata::from_toml(content))
                .transpose()?;
            let metadata = metadata.map(|metadata| metadata.extra).unwrap_or_default();
            let file_content = std::fs::read(path)?;
            let asset = ShaderSource::process(path, file_content, &metadata, gfx_bridge)?;
            Ok(asset.into())
        }
        AssetType::Texture => {
            let metadata = metadata_content
                .map(|content| Metadata::from_toml(content))
                .transpose()?;
            let metadata = metadata.map(|metadata| metadata.extra).unwrap_or_default();
            let file_content = std::fs::read(path)?;
            let asset = TextureSource::process(path, file_content, &metadata, gfx_bridge)?;
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
        "mat" => Ok(AssetType::Material),
        "gltf" | "glb" | "fbx" | "obj" | "3ds" | "blender" => Ok(AssetType::Model),
        "pmx" => Ok(AssetType::Model),
        "png" | "jpg" | "jpeg" | "gif" | "tif" | "tiff" | "tga" | "bmp" | "webp" => {
            Ok(AssetType::Texture)
        }
        "wgsl" => Ok(AssetType::Shader),
        _ => Err(AssetTypeDeduceError::UnsupportedExtension(
            path.to_path_buf(),
        )),
    }
}
