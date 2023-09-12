use super::TextureHandle;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteTexelMapping {
    pub x_min: u16,
    pub x_max: u16,
    pub y_min: u16,
    pub y_max: u16,
}

impl SpriteTexelMapping {
    pub fn new(x_min: u16, x_max: u16, y_min: u16, y_max: u16) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn min(self) -> (u16, u16) {
        (self.x_min, self.y_min)
    }

    pub fn max(self) -> (u16, u16) {
        (self.x_max, self.y_max)
    }

    pub fn width(self) -> u16 {
        u16::abs_diff(self.x_min, self.x_max)
    }

    pub fn height(self) -> u16 {
        u16::abs_diff(self.y_min, self.y_max)
    }
}
