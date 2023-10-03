use crate::{Asset, AssetDepsProvider, AssetType, GfxBridge};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("missing dependency: `{expected_id}` of type `{expected_ty}`")]
    MissingDependency {
        expected_id: Uuid,
        expected_ty: AssetType,
    },
    #[error("dependency type mismatch: `{expected_id}` of type `{expected_ty}`, but found `{actual_ty}`")]
    DependencyTypeMismatch {
        expected_id: Uuid,
        expected_ty: AssetType,
        actual_ty: AssetType,
    },
    #[error("{0}")]
    Other(String),
}

/// Represents an asset source, which contains all data required to construct an asset.
pub trait AssetSource: Serialize + for<'de> Deserialize<'de> {
    /// The asset type that this source can load.
    type Asset: ?Sized + Asset;

    /// List all dependencies of the asset.
    fn dependencies(&self) -> Vec<Uuid>;

    /// Constructs an asset from the source. The given id is the id of the asset.
    fn load(
        self,
        id: Uuid,
        deps_provider: &dyn AssetDepsProvider,
        gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError>;
}
