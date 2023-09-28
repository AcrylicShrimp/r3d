use super::{Texture, TextureAddressMode, TextureFilterMode};
use crate::{Asset, AssetDepsProvider, AssetLoadError, AssetSource, AssetType, TypedAsset};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteTexelRange {
    pub min: u16,
    pub max: u16,
}

pub trait SpriteAsset: Asset {
    fn texture(&self) -> &Texture;
    fn filter_mode(&self) -> TextureFilterMode;
    fn address_mode(&self) -> (TextureAddressMode, TextureAddressMode);
    fn texel_mapping(&self) -> (SpriteTexelRange, SpriteTexelRange);
}

#[derive(Serialize, Deserialize)]
pub struct SpriteSource {
    pub texture: Uuid,
    pub filter_mode: Option<TextureFilterMode>,
    pub address_mode: Option<(TextureAddressMode, TextureAddressMode)>,
    pub texel_mapping: (SpriteTexelRange, SpriteTexelRange),
}

impl AssetSource for SpriteSource {
    type Asset = dyn SpriteAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        vec![self.texture]
    }

    fn load(
        self,
        id: Uuid,
        deps_provider: &dyn AssetDepsProvider,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        let texture = deps_provider.find_dependency(self.texture).ok_or_else(|| {
            AssetLoadError::MissingDependency {
                expected_id: self.texture,
                expected_ty: AssetType::Texture,
            }
        })?;
        let texture =
            texture
                .as_texture()
                .ok_or_else(|| AssetLoadError::DependencyTypeMismatch {
                    expected_id: self.texture,
                    expected_ty: AssetType::Texture,
                    actual_ty: texture.ty(),
                })?;

        Ok(Arc::new(Sprite {
            id,
            texture: texture.clone(),
            filter_mode: self.filter_mode,
            address_mode: self.address_mode,
            texel_mapping: self.texel_mapping,
        }))
    }
}

struct Sprite {
    id: Uuid,
    texture: Texture,
    filter_mode: Option<TextureFilterMode>,
    address_mode: Option<(TextureAddressMode, TextureAddressMode)>,
    texel_mapping: (SpriteTexelRange, SpriteTexelRange),
}

impl Asset for Sprite {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Sprite(self)
    }
}

impl SpriteAsset for Sprite {
    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn filter_mode(&self) -> TextureFilterMode {
        self.filter_mode
            .unwrap_or_else(|| self.texture.filter_mode())
    }

    fn address_mode(&self) -> (TextureAddressMode, TextureAddressMode) {
        self.address_mode
            .unwrap_or_else(|| self.texture.address_mode())
    }

    fn texel_mapping(&self) -> (SpriteTexelRange, SpriteTexelRange) {
        self.texel_mapping
    }
}
