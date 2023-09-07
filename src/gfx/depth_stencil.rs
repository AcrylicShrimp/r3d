use super::GfxContextHandle;
use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};
use winit::dpi::PhysicalSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepthStencilMode {
    None,
    DepthOnly,
    DepthStencil,
}

impl DepthStencilMode {
    pub fn as_label_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::DepthOnly => "depth texture",
            Self::DepthStencil => "depth and stencil texture",
        }
    }

    pub fn as_texture_format(self) -> Option<TextureFormat> {
        match self {
            DepthStencilMode::None => None,
            DepthStencilMode::DepthOnly => Some(TextureFormat::Depth32Float),
            DepthStencilMode::DepthStencil => Some(TextureFormat::Depth24PlusStencil8),
        }
    }
}

pub struct DepthStencil {
    gfx_ctx: GfxContextHandle,
    mode: DepthStencilMode,
    texture: Option<Texture>,
    texture_view: Option<TextureView>,
}

impl DepthStencil {
    pub fn new(
        gfx_ctx: GfxContextHandle,
        mode: DepthStencilMode,
        size: PhysicalSize<u32>,
    ) -> Option<Self> {
        if size.width == 0 || size.height == 0 {
            return None;
        }

        let (texture, texture_view) = create_texture_and_view(&gfx_ctx.device, mode, size);
        Some(Self {
            gfx_ctx,
            mode,
            texture,
            texture_view,
        })
    }

    pub fn mode(&self) -> DepthStencilMode {
        self.mode
    }

    pub fn texture(&self) -> Option<&Texture> {
        self.texture.as_ref()
    }

    pub fn texture_view(&self) -> Option<&TextureView> {
        self.texture_view.as_ref()
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        let (texture, texture_view) =
            create_texture_and_view(&self.gfx_ctx.device, self.mode, size);
        self.texture = texture;
        self.texture_view = texture_view;
    }
}

fn create_texture_and_view(
    device: &Device,
    mode: DepthStencilMode,
    size: PhysicalSize<u32>,
) -> (Option<Texture>, Option<TextureView>) {
    match mode.as_texture_format() {
        Some(format) => {
            let texture = create_texture(device, mode, size, format);
            let texture_view = texture.create_view(&Default::default());
            (Some(texture), Some(texture_view))
        }
        None => (None, None),
    }
}

fn create_texture(
    device: &Device,
    mode: DepthStencilMode,
    size: PhysicalSize<u32>,
    format: TextureFormat,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some(mode.as_label_str()),
        size: Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[format],
    })
}
