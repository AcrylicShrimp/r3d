use crate::gfx::{BindGroupLayoutCache, FontHandle, SpriteTexelMapping, Texture, TextureHandle};
use std::{cmp::max, sync::Arc};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue,
    SamplerBindingType, ShaderStages, TextureAspect, TextureFormat, TextureSampleType,
    TextureViewDimension,
};

pub struct GlyphTexture {
    texture_bind_group: Arc<BindGroup>,
    sampler_bind_group: Arc<BindGroup>,
    texture: TextureHandle,
    font: FontHandle,
    offset_x: u16,
    offset_y: u16,
    line_height: u16,
}

impl GlyphTexture {
    pub fn new(
        device: &Device,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
        font: FontHandle,
    ) -> Self {
        let texture = Texture::create_empty(2048u16, 2048u16, TextureFormat::R8Unorm, device);
        let texture_bind_group_layout =
            bind_group_layout_cache.create_layout(vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }]);
        let sampler_bind_group_layout =
            bind_group_layout_cache.create_layout(vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            }]);
        let texture_bind_group = device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: texture_bind_group_layout.as_ref(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                }],
            })
            .into();
        let sampler_bind_group = device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: sampler_bind_group_layout.as_ref(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&texture.sampler),
                }],
            })
            .into();

        Self {
            texture_bind_group,
            sampler_bind_group,
            texture: TextureHandle::new(texture),
            font,
            offset_x: 0,
            offset_y: 0,
            line_height: 0,
        }
    }

    pub fn texture_bind_group(&self) -> &Arc<BindGroup> {
        &self.texture_bind_group
    }

    pub fn sampler_bind_group(&self) -> &Arc<BindGroup> {
        &self.sampler_bind_group
    }

    pub fn font(&self) -> &FontHandle {
        &self.font
    }

    pub fn texture(&self) -> &TextureHandle {
        &self.texture
    }

    pub fn glyph(
        &mut self,
        queue: &Queue,
        sdf_width: u16,
        sdf_height: u16,
        sdf: &[u8],
    ) -> Option<SpriteTexelMapping> {
        if 2048 < self.offset_y + sdf_height {
            return None;
        }

        if 2048 < self.offset_x + sdf_width {
            self.offset_x = 0;
            self.offset_y += self.line_height;
            self.line_height = sdf_height;

            if 2048 < self.offset_y + sdf_height {
                return None;
            }
        }

        let mapping = SpriteTexelMapping::new(
            self.offset_x as _,
            (self.offset_x + sdf_width) as _,
            self.offset_y as _,
            (self.offset_y + sdf_height) as _,
        );
        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: self.offset_x as u32,
                    y: self.offset_y as u32,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &sdf,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(sdf_width as u32),
                rows_per_image: Some(sdf_height as u32),
            },
            Extent3d {
                width: sdf_width as u32,
                height: sdf_height as u32,
                ..Default::default()
            },
        );

        self.offset_x += sdf_width;
        self.line_height = max(self.line_height, sdf_height);

        Some(mapping)
    }
}
