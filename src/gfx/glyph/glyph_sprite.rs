use crate::gfx::{SpriteTexelMapping, TextureHandle};
use std::fmt::Display;

#[derive(Clone)]
pub struct GlyphSprite {
    texture: TextureHandle,
    mapping: SpriteTexelMapping,
}

impl GlyphSprite {
    pub fn new(texture: TextureHandle, mapping: SpriteTexelMapping) -> Self {
        Self { texture, mapping }
    }

    pub fn texture(&self) -> &TextureHandle {
        &self.texture
    }

    pub fn mapping(&self) -> SpriteTexelMapping {
        self.mapping
    }

    pub fn width(&self) -> u32 {
        self.mapping.width() as u32
    }

    pub fn height(&self) -> u32 {
        self.mapping.height() as u32
    }
}

impl Display for GlyphSprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GlyphSprite({}x{})", self.width(), self.height())
    }
}
