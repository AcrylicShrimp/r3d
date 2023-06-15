use itertools::Itertools;
use thiserror::Error;
use wgpu::{
    Adapter, Backend, Backends, CompositeAlphaMode, CreateSurfaceError, Device, DeviceDescriptor,
    DeviceType, Features, Instance, InstanceDescriptor, PresentMode, Queue, RequestDeviceError,
    Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::window::Window;

#[derive(Error, Debug)]
pub enum GfxContextCreationError {
    #[error("failed to create surface")]
    CreateSurfaceError(#[from] CreateSurfaceError),
    #[error("no adapter found")]
    AdapterNotFound,
    #[error("failed to obtain device")]
    RequestDeviceError(#[from] RequestDeviceError),
}

pub struct GfxContext {
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub surface_config: SurfaceConfiguration,
}

impl GfxContext {
    pub(crate) async fn new(window: &Window) -> Result<Self, GfxContextCreationError> {
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
                        wgpu::Limits::downlevel_defaults()
                    },
                },
                None,
            )
            .await?;

        let window_inner_size = window.inner_size();
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: window_inner_size.width,
            height: window_inner_size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![TextureFormat::Bgra8Unorm],
        };
        surface.configure(&device, &surface_config);

        Ok(GfxContext {
            instance,
            device,
            queue,
            surface,
            surface_config,
        })
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
