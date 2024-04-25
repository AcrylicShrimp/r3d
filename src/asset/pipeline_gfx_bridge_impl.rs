use crate::ContextHandle;
use asset::assets::{
    SemanticShaderBindingKey, SemanticShaderInputKey, SemanticShaderOutputKey, ShaderGlobalItemKind,
};
use asset_pipeline::PipelineGfxBridge;
use wgpu::{BindingType, VertexFormat, VertexStepMode};

pub struct PipelineGfxBridgeImpl {
    context: ContextHandle,
}

impl PipelineGfxBridgeImpl {
    pub fn new(context: ContextHandle) -> Self {
        Self { context }
    }
}

impl PipelineGfxBridge for PipelineGfxBridgeImpl {
    fn get_semantic_binding_key(
        &self,
        name: &str,
        kind: &ShaderGlobalItemKind,
    ) -> Option<SemanticShaderBindingKey> {
        let shader_mgr = self.context.shader_mgr();
        let key = shader_mgr.find_semantic_binding(name)?;
        let binding = shader_mgr.get_semantic_binding(key).unwrap();
        let is_matching = match (binding.ty, kind) {
            (
                BindingType::Buffer {
                    min_binding_size, ..
                },
                ShaderGlobalItemKind::Buffer { size },
            ) => match min_binding_size {
                Some(min_binding_size) => min_binding_size == *size,
                None => true,
            },
            (
                wgpu::BindingType::Texture {
                    sample_type,
                    view_dimension,
                    multisampled,
                },
                ShaderGlobalItemKind::Texture {
                    sample_type: item_sample_type,
                    view_dimension: item_view_dimension,
                    multisampled: item_multisampled,
                    ..
                },
            ) => {
                sample_type == *item_sample_type
                    && view_dimension == *item_view_dimension
                    && multisampled == *item_multisampled
            }
            (
                wgpu::BindingType::Sampler(binding_type),
                ShaderGlobalItemKind::Sampler {
                    binding_type: item_binding_type,
                },
            ) => binding_type == *item_binding_type,
            _ => false,
        };

        match is_matching {
            true => Some(SemanticShaderBindingKey::new(key.get())),
            false => None,
        }
    }

    fn get_semantic_input_key(
        &self,
        name: &str,
        step_mode: VertexStepMode,
        format: VertexFormat,
    ) -> Option<SemanticShaderInputKey> {
        let shader_mgr = self.context.shader_mgr();
        let key = shader_mgr.find_semantic_input(name)?;
        let input = shader_mgr.get_semantic_input(key).unwrap();
        let is_matching = { input.step_mode == step_mode && input.format == format };

        match is_matching {
            true => Some(SemanticShaderInputKey::new(key.get())),
            false => None,
        }
    }

    fn get_semantic_output_key(
        &self,
        name: &str,
        location: u32,
    ) -> Option<SemanticShaderOutputKey> {
        let shader_mgr = self.context.shader_mgr();
        let key = shader_mgr.find_semantic_output(name)?;
        let output = shader_mgr.get_semantic_output(key).unwrap();
        let is_matching = { output.location == location };

        match is_matching {
            true => Some(SemanticShaderOutputKey::new(key.get())),
            false => None,
        }
    }
}
