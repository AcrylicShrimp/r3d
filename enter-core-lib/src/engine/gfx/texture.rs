use codegen::Handle;
use wgpu::{Sampler, TextureView};

#[derive(Handle)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub width: u16,
    pub height: u16,
}
