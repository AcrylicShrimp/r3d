use crate::ContextHandle;
use asset::{
    assets::{TextureAddressMode, TextureFilterMode, TextureFormat},
    GfxBridge, GfxBuffer, GfxSampler, GfxShaderModule, GfxTexture, GfxTextureView,
};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, Extent3d, FilterMode, ImageCopyTexture,
    ImageDataLayout, Origin3d, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureUsages,
};

pub struct GfxBridgeImpl {
    context: ContextHandle,
}

impl GfxBridgeImpl {
    pub fn new(context: ContextHandle) -> Self {
        Self { context }
    }
}

impl GfxBridge for GfxBridgeImpl {
    fn upload_vertex_buffer(&self, usage: BufferUsages, content: &[u8]) -> GfxBuffer {
        let buffer = self
            .context
            .gfx_ctx
            .device
            .create_buffer(&BufferDescriptor {
                label: None,
                size: content.len() as BufferAddress,
                usage,
                mapped_at_creation: false,
            });
        self.context.gfx_ctx.queue.write_buffer(&buffer, 0, content);

        GfxBuffer::new(buffer)
    }

    fn compile_shader(&self, source: ShaderSource) -> GfxShaderModule {
        let shader = self
            .context
            .gfx_ctx
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source,
            });

        GfxShaderModule::new(shader)
    }

    fn upload_texture(
        &self,
        width: u16,
        height: u16,
        format: TextureFormat,
        texels: &[u8],
    ) -> asset::GfxTexture {
        let format = match format {
            TextureFormat::RGBA8 => wgpu::TextureFormat::Rgba8Unorm,
        };
        let texture = self
            .context
            .gfx_ctx
            .device
            .create_texture(&TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[format],
            });
        self.context.gfx_ctx.queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            texels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width as u32),
                rows_per_image: Some(height as u32),
            },
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );

        GfxTexture::new(texture)
    }

    fn create_texture_view(&self, texture: &Texture) -> GfxTextureView {
        GfxTextureView::new(texture.create_view(&Default::default()))
    }

    fn create_sampler(
        &self,
        filter_mode: TextureFilterMode,
        address_mode: (TextureAddressMode, TextureAddressMode),
    ) -> GfxSampler {
        let (texel_filter_mode, mipmap_filter_mode) = match filter_mode {
            TextureFilterMode::Point => (FilterMode::Nearest, FilterMode::Nearest),
            TextureFilterMode::Bilinear => (FilterMode::Linear, FilterMode::Nearest),
            TextureFilterMode::Trilinear => (FilterMode::Linear, FilterMode::Linear),
        };
        let (address_mode_u, address_mode_v) = address_mode;
        let address_mode_u = convert_address_mode(address_mode_u);
        let address_mode_v = convert_address_mode(address_mode_v);
        let sampler = self
            .context
            .gfx_ctx
            .device
            .create_sampler(&SamplerDescriptor {
                label: None,
                address_mode_u,
                address_mode_v,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: texel_filter_mode,
                min_filter: texel_filter_mode,
                mipmap_filter: mipmap_filter_mode,
                lod_min_clamp: 0.0,
                lod_max_clamp: 32.0,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            });

        GfxSampler::new(sampler)
    }
}

fn convert_address_mode(mode: TextureAddressMode) -> wgpu::AddressMode {
    match mode {
        TextureAddressMode::Repeat => wgpu::AddressMode::Repeat,
        TextureAddressMode::Clamp => wgpu::AddressMode::ClampToEdge,
    }
}
