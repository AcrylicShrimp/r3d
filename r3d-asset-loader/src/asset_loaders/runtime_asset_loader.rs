use crate::{AssetDatabase, AssetLoadError, AssetLoader};
use asset::{AssetKey, AssetSource, GfxBridge, TypedAsset};
use asset_pipeline::{
    deduce_asset_type_from_path, process_asset, PipelineGfxBridge, TypedAssetSource,
};
use std::collections::HashMap;

pub struct RuntimeAssetLoader {
    gfx_bridge: Box<dyn GfxBridge>,
    pipeline_gfx_bridge: Box<dyn PipelineGfxBridge>,
}

impl RuntimeAssetLoader {
    pub fn new(
        gfx_bridge: impl GfxBridge + 'static,
        pipeline_gfx_bridge: impl PipelineGfxBridge + 'static,
    ) -> Self {
        Self {
            gfx_bridge: Box::new(gfx_bridge),
            pipeline_gfx_bridge: Box::new(pipeline_gfx_bridge),
        }
    }
}

impl AssetLoader for RuntimeAssetLoader {
    fn load_asset(
        &self,
        key: &AssetKey,
        database: &AssetDatabase,
    ) -> Result<TypedAsset, AssetLoadError> {
        let processed = match key {
            AssetKey::Id(id) => {
                let id = *id;
                let data = database
                    .find_asset_by_id(id)
                    .ok_or_else(|| AssetLoadError::AssetNotFound(id))?;
                let processed = process_asset(
                    &data.path,
                    data.asset_type,
                    Some(&data.metadata_content),
                    &*self.pipeline_gfx_bridge,
                )?;

                processed
            }
            AssetKey::Path(path) => {
                let asset_type = deduce_asset_type_from_path(path)?;
                let processed = process_asset(
                    path,
                    asset_type,
                    None as Option<&str>,
                    &*self.pipeline_gfx_bridge,
                )?;

                processed
            }
        };

        // Resolve dependencies. NOTE: It can be recursive.
        let deps = match &processed {
            TypedAssetSource::Font(source) => source.dependencies(),
            TypedAssetSource::Material(source) => source.dependencies(),
            TypedAssetSource::Model(source) => source.dependencies(),
            TypedAssetSource::Shader(source) => source.dependencies(),
            TypedAssetSource::Texture(source) => source.dependencies(),
        };
        let deps = deps
            .into_iter()
            .map(|key| {
                let asset = self.load_asset(&key, database)?;

                Ok((key, asset))
            })
            .collect::<Result<HashMap<_, _>, AssetLoadError>>()?;

        Ok(match processed {
            TypedAssetSource::Font(source) => {
                TypedAsset::Font(source.load(key.clone(), &deps, &*self.gfx_bridge)?)
            }
            TypedAssetSource::Material(source) => {
                TypedAsset::Material(source.load(key.clone(), &deps, &*self.gfx_bridge)?)
            }
            TypedAssetSource::Model(source) => {
                TypedAsset::Model(source.load(key.clone(), &deps, &*self.gfx_bridge)?)
            }
            TypedAssetSource::Shader(source) => {
                TypedAsset::Shader(source.load(key.clone(), &deps, &*self.gfx_bridge)?)
            }
            TypedAssetSource::Texture(source) => {
                TypedAsset::Texture(source.load(key.clone(), &deps, &*self.gfx_bridge)?)
            }
        })
    }
}
