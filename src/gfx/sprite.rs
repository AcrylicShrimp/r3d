use super::{SpriteTexelMapping, TextureHandle};
use codegen::Handle;

#[derive(Handle)]
pub struct Sprite {
    texture: TextureHandle,
    mapping: SpriteTexelMapping,
}

impl Sprite {
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
