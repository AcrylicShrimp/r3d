use asset::assets::{
    SemanticShaderBindingKey, SemanticShaderInputKey, SemanticShaderOutputKey, ShaderGlobalItemKind,
};
use wgpu::{VertexFormat, VertexStepMode};

/// A bridge interface to interact with the GPU.
/// This bridge is used in the asset pipeline to generate assets.
/// In other words, it is not used in the runtime asset loading.
pub trait PipelineGfxBridge {
    /// Gets the semantic binding key of a global item.
    fn get_semantic_binding_key(
        &self,
        name: &str,
        kind: &ShaderGlobalItemKind,
    ) -> Option<SemanticShaderBindingKey>;
    /// Gets the semantic input key of a input field.
    fn get_semantic_input_key(
        &self,
        name: &str,
        step_mode: VertexStepMode,
        format: VertexFormat,
    ) -> Option<SemanticShaderInputKey>;
    /// Gets the semantic output key of a output item.
    fn get_semantic_output_key(&self, name: &str, location: u32)
        -> Option<SemanticShaderOutputKey>;
}
