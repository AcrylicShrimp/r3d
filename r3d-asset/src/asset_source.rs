use crate::{Asset, AssetDepsProvider, AssetKey, AssetType, GfxBridge};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("missing dependency: `{expected_key}` of type `{expected_ty}`")]
    MissingDependency {
        expected_key: AssetKey,
        expected_ty: AssetType,
    },
    #[error("dependency type mismatch: `{expected_key}` of type `{expected_ty}`, but found `{actual_ty}`")]
    DependencyTypeMismatch {
        expected_key: AssetKey,
        expected_ty: AssetType,
        actual_ty: AssetType,
    },
    #[error(
        "invalid sprite name: texture `{texture_key}` does not contain sprite `{sprite_name}`"
    )]
    InvalidSpriteName {
        texture_key: AssetKey,
        sprite_name: String,
    },
    #[error(
        "invalid nine-patch name: texture `{texture_key}` does not contain nine-patch `{nine_patch_name}`"
    )]
    InvalidNinePatchName {
        texture_key: AssetKey,
        nine_patch_name: String,
    },
    #[error("{0}")]
    Other(String),
}

/// Represents an asset source, which contains all data required to construct an asset.
pub trait AssetSource: Serialize + for<'de> Deserialize<'de> {
    /// The asset type that this source can load.
    type Asset: ?Sized + Asset;

    /// List all dependencies of the asset.
    fn dependencies(&self) -> Vec<AssetKey>;

    /// Constructs an asset from the source. The given id is the id of the asset.
    fn load(
        self,
        key: AssetKey,
        deps_provider: &dyn AssetDepsProvider,
        gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError>;
}
