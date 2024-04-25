use super::GfxContextHandle;

pub struct GPUResourceManager {
    gfx_ctx: GfxContextHandle,
}

impl GPUResourceManager {
    pub fn new(gfx_ctx: GfxContextHandle) -> Self {
        Self { gfx_ctx }
    }
}
