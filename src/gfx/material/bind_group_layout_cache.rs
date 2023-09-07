use crate::gfx::GfxContextHandle;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Weak},
};
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Device};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindGroupLayoutKey {
    pub entries: Vec<BindGroupLayoutEntry>,
}

impl BindGroupLayoutKey {
    pub fn new(entries: Vec<BindGroupLayoutEntry>) -> Self {
        Self { entries }
    }

    pub fn create_bind_group_layout(&self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &self.entries,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CachedBindGroupLayout {
    key: BindGroupLayoutKey,
    layout: Arc<BindGroupLayout>,
}

impl CachedBindGroupLayout {
    pub fn new(key: BindGroupLayoutKey, layout: Arc<BindGroupLayout>) -> Self {
        Self { key, layout }
    }

    pub fn key(&self) -> &BindGroupLayoutKey {
        &self.key
    }
}

impl AsRef<BindGroupLayout> for CachedBindGroupLayout {
    fn as_ref(&self) -> &BindGroupLayout {
        &self.layout
    }
}

impl PartialEq for CachedBindGroupLayout {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.layout, &other.layout)
    }
}

impl Eq for CachedBindGroupLayout {}

impl Hash for CachedBindGroupLayout {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.layout).hash(state);
    }
}

pub struct BindGroupLayoutCache {
    gfx_ctx: GfxContextHandle,
    caches: HashMap<BindGroupLayoutKey, Weak<BindGroupLayout>>,
}

// TODO: Provide a way to drop unused bind group layouts.
impl BindGroupLayoutCache {
    pub fn new(gfx_ctx: GfxContextHandle) -> Self {
        Self {
            gfx_ctx,
            caches: HashMap::new(),
        }
    }

    pub fn create_layout(&mut self, entries: Vec<BindGroupLayoutEntry>) -> CachedBindGroupLayout {
        let key = BindGroupLayoutKey::new(entries);

        if let Some(layout) = self.caches.get(&key).and_then(|weak| weak.upgrade()) {
            return CachedBindGroupLayout::new(key, layout);
        }

        let layout = Arc::new(key.create_bind_group_layout(&self.gfx_ctx.device));
        self.caches.insert(key.clone(), Arc::downgrade(&layout));

        CachedBindGroupLayout::new(key, layout)
    }
}
