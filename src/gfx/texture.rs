use codegen::Handle;
use image::{DynamicImage, GenericImageView};
use std::sync::Arc;
use wgpu::{
    util::DeviceExt, Device, Extent3d, Queue, Sampler, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView,
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
        let sampler = device.create_sampler(&Default::default());

        Self {
            texture: texture.into(),
            view: view.into(),
            sampler: sampler.into(),
            width: width as u16,
            height: height as u16,
        }
    }
}
