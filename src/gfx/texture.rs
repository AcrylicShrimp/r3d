use codegen::Handle;
use image::{DynamicImage, GenericImageView};
use std::sync::Arc;
use wgpu::{
    util::DeviceExt, AddressMode, Device, Extent3d, FilterMode, Queue, Sampler, SamplerDescriptor,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

#[derive(Handle)]
pub struct Texture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<TextureView>,
    pub sampler: Arc<Sampler>,
    pub width: u16,
    pub height: u16,
}

impl Texture {
    pub fn from_image(
        format: TextureFormat,
        image: &DynamicImage,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let (width, height) = image.dimensions();
        let texture_extent = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: None,
                size: texture_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[format],
            },
            image.as_bytes(),
        );
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        Self {
            texture: texture.into(),
            view: view.into(),
            sampler: sampler.into(),
            width: width as u16,
            height: height as u16,
        }
    }

    pub fn create_empty(width: u16, height: u16, format: TextureFormat, device: &Device) -> Self {
        let texture_extent = Extent3d {
            width: width as _,
            height: height as _,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        Self {
            texture: texture.into(),
            view: view.into(),
            sampler: sampler.into(),
            width,
            height,
        }
    }
}
