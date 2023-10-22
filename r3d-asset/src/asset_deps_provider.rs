use crate::{AssetKey, TypedAsset};
use std::collections::HashMap;

/// Provides asset dependencies. All assets marked as dependencies can be fetched from this provider.
pub trait AssetDepsProvider {
    /// Find a dependency by its id. Returns `None` if the dependency is not found.
    /// All dependencies must be specified in the [`crate::AssetSource::dependencies()`].
    fn find_dependency(&self, key: &AssetKey) -> Option<TypedAsset>;
}

impl AssetDepsProvider for HashMap<AssetKey, TypedAsset> {
    fn find_dependency(&self, key: &AssetKey) -> Option<TypedAsset> {
        self.get(key).cloned()
    }
}
