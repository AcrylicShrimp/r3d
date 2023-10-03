use crate::{Asset, AssetDepsProvider, AssetLoadError, AssetSource, GfxBridge, TypedAsset};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    RGB8,
    RGBA8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFilterMode {
    Point,
    Bilinear,
    Trilinear,
}

impl TextureFilterMode {
    pub fn needs_mipmap(&self) -> bool {
        match self {
            TextureFilterMode::Point => false,
            TextureFilterMode::Bilinear => false,
            TextureFilterMode::Trilinear => true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureAddressMode {
    Clamp,
    Repeat,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteTexelRange {
    pub min: u16,
    pub max: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NinePatchTexelRange {
    pub min: u16,
    pub mid_min: u16,
    pub mid_max: u16,
    pub max: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sprite {
    pub name: String,
    pub filter_mode: TextureFilterMode,
    pub address_mode: (TextureAddressMode, TextureAddressMode),
    pub texel_mapping: (SpriteTexelRange, SpriteTexelRange),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NinePatch {
    pub name: String,
    pub filter_mode: TextureFilterMode,
    pub address_mode: (TextureAddressMode, TextureAddressMode),
    pub texel_mapping: (NinePatchTexelRange, NinePatchTexelRange),
}

/// Represents a texture asset. It supplies texture parameters too.
pub trait TextureAsset: Asset {
    fn handle(&self) -> &wgpu::Texture;
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn format(&self) -> TextureFormat;
    fn filter_mode(&self) -> TextureFilterMode;
    fn address_mode(&self) -> (TextureAddressMode, TextureAddressMode);
    fn sprites(&self) -> &[Sprite];
    fn nine_patches(&self) -> &[NinePatch];
}

#[derive(Serialize, Deserialize)]
pub struct TextureSource {
    pub width: u16,
    pub height: u16,
    pub format: TextureFormat,
    pub filter_mode: TextureFilterMode,
    pub address_mode: (TextureAddressMode, TextureAddressMode),
    pub texels: Vec<u8>,
    pub sprites: Vec<Sprite>,
    pub nine_patches: Vec<NinePatch>,
}

impl AssetSource for TextureSource {
    type Asset = dyn TextureAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }

    fn load(
        self,
        id: Uuid,
        _deps_provider: &dyn AssetDepsProvider,
        gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        Ok(Arc::new(Texture {
            id,
            handle: gfx_bridge.upload_texture(
                self.width,
                self.height,
                self.format,
                self.filter_mode.needs_mipmap(),
                &self.texels,
            ),
            width: self.width,
            height: self.height,
            format: self.format,
            filter_mode: self.filter_mode,
            address_mode: self.address_mode,
            sprites: self.sprites,
            nine_patches: self.nine_patches,
        }))
    }
}

struct Texture {
    id: Uuid,
    handle: wgpu::Texture,
    width: u16,
    height: u16,
    format: TextureFormat,
    filter_mode: TextureFilterMode,
    address_mode: (TextureAddressMode, TextureAddressMode),
    sprites: Vec<Sprite>,
    nine_patches: Vec<NinePatch>,
}

impl Asset for Texture {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Texture(self)
    }
}

impl TextureAsset for Texture {
    fn handle(&self) -> &wgpu::Texture {
        &self.handle
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn format(&self) -> TextureFormat {
        self.format
    }

    fn filter_mode(&self) -> TextureFilterMode {
        self.filter_mode
    }

    fn address_mode(&self) -> (TextureAddressMode, TextureAddressMode) {
        self.address_mode
    }

    fn sprites(&self) -> &[Sprite] {
        &self.sprites
    }

    fn nine_patches(&self) -> &[NinePatch] {
        &self.nine_patches
    }
}
