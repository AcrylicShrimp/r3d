use crate::{AssetDatabase, AssetLoadError, AssetLoader};
use asset::{AssetSource, TypedAsset};
use asset_pipeline::{process_asset, TypedAssetSource};
use std::collections::HashMap;
use uuid::Uuid;

pub struct RawFileAssetLoader;

impl AssetLoader for RawFileAssetLoader {
    fn load_asset(&self, id: Uuid, database: &AssetDatabase) -> Result<TypedAsset, AssetLoadError> {
        let data = database
            .find_asset_by_id(id)
            .ok_or_else(|| AssetLoadError::AssetNotFound(id))?;
        let processed = process_asset(&data.path, data.asset_type, &data.metadata_content)?;

        // Resolve dependencies. NOTE: It can be recursive.
        let deps = match &processed {
            TypedAssetSource::Font(source) => source.dependencies(),
            TypedAssetSource::Texture(source) => source.dependencies(),
        };
        let deps = deps
            .into_iter()
            .map(|id| Ok((id, self.load_asset(id, database)?)))
            .collect::<Result<HashMap<_, _>, AssetLoadError>>()?;

        Ok(match processed {
            TypedAssetSource::Font(source) => TypedAsset::Font(source.load(id, &deps)?),
            TypedAssetSource::Texture(source) => TypedAsset::Texture(source.load(id, &deps)?),
        })
    }
}
