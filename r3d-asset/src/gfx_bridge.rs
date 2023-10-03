use crate::assets::TextureFormat;

/// A bridge interface to interact with the GPU.
/// This bridge is used in runtime asset loading to obtain GPU resource handles.
pub trait GfxBridge {
    /// Uploads a texture to the GPU and returns a handle to it.
    fn upload_texture(
        &self,
        width: u16,
        height: u16,
        format: TextureFormat,
        generate_mipmaps: bool,
        texels: &[u8],
    ) -> wgpu::Texture;
}
