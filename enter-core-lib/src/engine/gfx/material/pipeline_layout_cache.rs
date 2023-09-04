use super::CachedBindGroupLayout;
use crate::engine::gfx::GfxContextHandle;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Weak},
};
use wgpu::{Device, PipelineLayout, PipelineLayoutDescriptor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineLayoutKey {
    pub bind_group_layouts: Vec<CachedBindGroupLayout>,
}

impl PipelineLayoutKey {
    pub fn new(bind_group_layouts: Vec<CachedBindGroupLayout>) -> Self {
        Self { bind_group_layouts }
    }

    pub fn create_pipeline_layout(&self, device: &Device) -> PipelineLayout {
        let layouts = Vec::from_iter(self.bind_group_layouts.iter().map(|layout| layout.as_ref()));
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        })
    }
}

#[derive(Debug, Clone)]
pub struct CachedPipelineLayout {
    layout: Arc<PipelineLayout>,
}

impl CachedPipelineLayout {
    pub fn new(layout: Arc<PipelineLayout>) -> Self {
        Self { layout }
    }
}

impl AsRef<PipelineLayout> for CachedPipelineLayout {
    fn as_ref(&self) -> &PipelineLayout {
        &self.layout
    }
}

impl PartialEq for CachedPipelineLayout {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.layout, &other.layout)
    }
}

impl Eq for CachedPipelineLayout {}

impl Hash for CachedPipelineLayout {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.layout).hash(state);
    }
}

pub struct PipelineLayoutCache {
    gfx_ctx: GfxContextHandle,
    caches: HashMap<PipelineLayoutKey, Weak<PipelineLayout>>,
}

impl PipelineLayoutCache {
    pub fn new(gfx_ctx: GfxContextHandle) -> Self {
        Self {
            gfx_ctx,
            caches: HashMap::new(),
        }
    }

    pub fn create_layout(
        &mut self,
        bind_group_layouts: Vec<CachedBindGroupLayout>,
    ) -> CachedPipelineLayout {
        let key = PipelineLayoutKey::new(bind_group_layouts);

        if let Some(layout) = self.caches.get(&key).and_then(|weak| weak.upgrade()) {
            return CachedPipelineLayout::new(layout);
        }

        let layout = Arc::new(key.create_pipeline_layout(&self.gfx_ctx.device));
        self.caches.insert(key, Arc::downgrade(&layout));

        CachedPipelineLayout::new(layout)
    }
}
