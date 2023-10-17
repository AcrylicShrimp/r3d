use codegen::Handle;
use itertools::Itertools;
use std::cell::RefCell;
use thiserror::Error;
use wgpu::{
    Adapter, Backend, Backends, CompositeAlphaMode, CreateSurfaceError, Device, DeviceDescriptor,
    DeviceType, Features, Instance, InstanceDescriptor, PresentMode, Queue, RequestDeviceError,
    Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::{dpi::PhysicalSize, window::Window};

mod built_in_shader_manager;
mod camera;
mod color;
mod depth_stencil;
mod font;
mod glyph;
mod material;
mod mesh;
mod nine_patch;
mod render_mgr;
mod renderer;
mod screen_mgr;
mod sprite;
mod texture;

pub use built_in_shader_manager::*;
pub use camera::*;
pub use color::*;
pub use depth_stencil::*;
pub use font::*;
pub use glyph::*;
pub use material::*;
pub use mesh::*;
pub use nine_patch::*;
pub use render_mgr::*;
pub use renderer::*;
pub use screen_mgr::*;
pub use sprite::*;
pub use texture::*;

#[derive(Error, Debug)]
pub enum GfxContextCreationError {
    #[error("failed to create surface")]
    CreateSurfaceError(#[from] CreateSurfaceError),
    #[error("no adapter found")]
    AdapterNotFound,
    #[error("failed to obtain device")]
    RequestDeviceError(#[from] RequestDeviceError),
}

#[derive(Handle)]
pub struct GfxContext {
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub surface_config: RefCell<SurfaceConfiguration>,
}

impl GfxContext {
    pub async fn new(window: &Window) -> Result<Self, GfxContextCreationError> {
        let instance = Instance::new(InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(window) }?;
        let adapters = instance
            .enumerate_adapters(Backends::all())
            .collect::<Vec<_>>();
        let adapter = if let Some(adapter_index) = select_adapter(&surface, &adapters) {
            &adapters[adapter_index]
        } else {
            return Err(GfxContextCreationError::AdapterNotFound);
        };

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::CLEAR_TEXTURE,
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await?;

        let window_inner_size = window.inner_size();
        let surface_config = RefCell::new(SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: window_inner_size.width,
            height: window_inner_size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![TextureFormat::Bgra8Unorm],
        });
        surface.configure(&device, &surface_config.borrow());

        Ok(GfxContext {
            instance,
            device,
            queue,
            surface,
            surface_config,
        })
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        let mut surface_config = self.surface_config.borrow_mut();
        surface_config.width = size.width;
        surface_config.height = size.height;
        self.surface.configure(&self.device, &surface_config);
    }
}

fn select_adapter(surface: &Surface, adapters: impl AsRef<[Adapter]>) -> Option<usize> {
    let adapters = adapters
        .as_ref()
        .iter()
        .filter(|adapter| !surface.get_capabilities(adapter).formats.is_empty())
        .collect::<Vec<_>>();

    if adapters.is_empty() {
        return None;
    }

    let mut scores = adapters.iter().map(|_| 0).collect::<Vec<_>>();

    for (index, adapter) in adapters.iter().enumerate() {
        if surface.get_capabilities(adapter).formats.is_empty() {
            continue;
        }

        let info = adapter.get_info();
        let device_score = match info.device_type {
            DeviceType::IntegratedGpu => 10,
            DeviceType::DiscreteGpu => 20,
            DeviceType::Cpu => -10,
            _ => 0,
        };
        let backend_score = match info.backend {
            // The Vulkan is available with other backends simultaneously on some platforms.
            // Because the dedicated backends are preferred over the Vulkan, we set the score of the Vulkan slightly lower than others.
            Backend::Metal => 2,
            Backend::Dx12 => 2,
            Backend::Vulkan => 1,
            _ => 0,
        };
        scores[index] += device_score + backend_score;
    }

    scores.iter().position_max()
}
