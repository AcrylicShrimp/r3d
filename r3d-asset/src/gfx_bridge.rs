use crate::assets::{TextureAddressMode, TextureFilterMode, TextureFormat};
use std::sync::Arc;
use wgpu::{BufferUsages, ShaderSource};

pub type GfxBuffer = Arc<wgpu::Buffer>;
pub type GfxShaderModule = Arc<wgpu::ShaderModule>;
pub type GfxTexture = Arc<wgpu::Texture>;
pub type GfxTextureView = Arc<wgpu::TextureView>;
pub type GfxSampler = Arc<wgpu::Sampler>;

/// A bridge interface to interact with the GPU.
/// This bridge is used in runtime asset loading to obtain GPU resource handles.
pub trait GfxBridge {
    /// Uploads a vertex buffer to the GPU and returns a handle to it.
    fn upload_vertex_buffer(&self, usage: BufferUsages, content: &[u8]) -> GfxBuffer;
    /// Compiles a shader and returns a handle to it.
    fn compile_shader(&self, source: ShaderSource) -> GfxShaderModule;
    /// Uploads a texture to the GPU and returns a handle to it.
    fn upload_texture(
        &self,
        width: u16,
        height: u16,
        format: TextureFormat,
        generate_mipmaps: bool,
        texels: &[u8],
    ) -> GfxTexture;
    /// Creates a texture view from a texture.
    fn create_texture_view(&self, texture: &wgpu::Texture) -> GfxTextureView;
    /// Creates a sampler.
    fn create_sampler(
        &self,
        filter_mode: TextureFilterMode,
        address_mode: (TextureAddressMode, TextureAddressMode),
    ) -> GfxSampler;
}
