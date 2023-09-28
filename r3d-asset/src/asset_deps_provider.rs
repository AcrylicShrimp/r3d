use crate::TypedAsset;
use std::collections::HashMap;
use uuid::Uuid;

/// Provides asset dependencies. All assets marked as dependencies can be fetched from this provider.
pub trait AssetDepsProvider {
    /// Find a dependency by its id. Returns `None` if the dependency is not found.
    /// All dependencies must be specified in the [`crate::AssetSource::dependencies()`].
    fn find_dependency(&self, id: Uuid) -> Option<TypedAsset>;
}

impl AssetDepsProvider for HashMap<Uuid, TypedAsset> {
    fn find_dependency(&self, id: Uuid) -> Option<TypedAsset> {
        self.get(&id).cloned()
    }
}
