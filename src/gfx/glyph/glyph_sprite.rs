use crate::gfx::{SpriteTexelMapping, TextureHandle};
use std::sync::Arc;
use wgpu::BindGroup;

#[derive(Clone)]
pub struct GlyphSprite {
    texture_bind_group: Arc<BindGroup>,
    sampler_bind_group: Arc<BindGroup>,
    texture: TextureHandle,
    mapping: SpriteTexelMapping,
}

impl GlyphSprite {
    pub fn new(
        texture_bind_group: Arc<BindGroup>,
        sampler_bind_group: Arc<BindGroup>,
        texture: TextureHandle,
        mapping: SpriteTexelMapping,
    ) -> Self {
        Self {
            texture_bind_group,
            sampler_bind_group,
            texture,
            mapping,
        }
    }

    pub fn texture_bind_group(&self) -> &Arc<BindGroup> {
        &self.texture_bind_group
    }

    pub fn sampler_bind_group(&self) -> &Arc<BindGroup> {
        &self.sampler_bind_group
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
