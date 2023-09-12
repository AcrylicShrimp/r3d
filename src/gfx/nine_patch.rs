use super::TextureHandle;
use codegen::Handle;

#[derive(Handle)]
pub struct NinePatch {
    texture: TextureHandle,
    mapping: NinePatchTexelMapping,
}

impl NinePatch {
    pub fn new(texture: TextureHandle, mapping: NinePatchTexelMapping) -> Self {
        Self { texture, mapping }
    }

    pub fn texture(&self) -> &TextureHandle {
        &self.texture
    }

    pub fn mapping(&self) -> NinePatchTexelMapping {
        self.mapping
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NinePatchTexelMapping {
    pub x_min: u16,
    pub x_mid_left: u16,
    pub x_mid_right: u16,
    pub x_max: u16,
    pub y_min: u16,
    pub y_mid_bottom: u16,
    pub y_mid_top: u16,
    pub y_max: u16,
}

impl NinePatchTexelMapping {
    pub fn new(
        x_min: u16,
        x_mid_left: u16,
        x_mid_right: u16,
        x_max: u16,
        y_min: u16,
        y_mid_bottom: u16,
        y_mid_top: u16,
        y_max: u16,
    ) -> Self {
        Self {
            x_min,
            x_mid_left,
            x_mid_right,
            x_max,
            y_min,
            y_mid_bottom,
            y_mid_top,
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

    pub fn mid_width(self) -> u16 {
        u16::abs_diff(self.x_mid_left, self.x_mid_right)
    }

    pub fn mid_height(self) -> u16 {
        u16::abs_diff(self.y_mid_bottom, self.y_mid_top)
    }
}
