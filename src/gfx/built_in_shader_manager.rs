use super::{BindGroupLayoutCache, ShaderHandle, ShaderManager};
use std::{collections::HashMap, num::NonZeroU64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuiltInShaderKey(NonZeroU64);

impl BuiltInShaderKey {
    pub const fn new(key: NonZeroU64) -> Self {
        Self(key)
    }
}

pub const BUILT_IN_SHADER_UI_ELEMENT_NORMAL: BuiltInShaderKey =
    BuiltInShaderKey::new(unsafe { NonZeroU64::new_unchecked(1) });
pub const BUILT_IN_SHADER_UI_TEXT_NORMAL: BuiltInShaderKey =
    BuiltInShaderKey::new(unsafe { NonZeroU64::new_unchecked(11) });

pub struct BuiltInShaderManager {
    shaders: HashMap<BuiltInShaderKey, ShaderHandle>,
}

impl BuiltInShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    pub fn init(
        &mut self,
        shader_mgr: &ShaderManager,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
    ) {
        self.add_shader(
            shader_mgr,
            bind_group_layout_cache,
            BUILT_IN_SHADER_UI_ELEMENT_NORMAL,
            include_str!("./built_in_shaders/ui_element.normal.wgsl"),
        );
        self.add_shader(
            shader_mgr,
            bind_group_layout_cache,
            BUILT_IN_SHADER_UI_TEXT_NORMAL,
            include_str!("./built_in_shaders/ui_text.normal.wgsl"),
        );
    }

    fn add_shader(
        &mut self,
        shader_mgr: &ShaderManager,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
        key: BuiltInShaderKey,
        source: &str,
    ) {
        let shader = shader_mgr
            .create_shader(bind_group_layout_cache, source)
            .unwrap();
        self.shaders.insert(key, shader);
    }

    pub fn find_shader(&self, key: BuiltInShaderKey) -> Option<ShaderHandle> {
        self.shaders.get(&key).cloned()
    }
}
