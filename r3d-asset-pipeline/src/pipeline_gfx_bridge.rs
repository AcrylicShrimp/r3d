use asset::assets::{SemanticShaderBindingKey, SemanticShaderInputKey, SemanticShaderOutputKey};
use naga::Module;
use wgpu::{VertexFormat, VertexStepMode};

/// A bridge interface to interact with the GPU.
/// This bridge is used in the asset pipeline to generate assets.
/// In other words, it is not used in the runtime asset loading.
pub trait PipelineGfxBridge {
    /// Gets the semantic binding key of a global item.
    fn get_semantic_binding_key(
        &self,
        module: &Module,
        name: &str,
    ) -> Option<SemanticShaderBindingKey>;
    /// Gets the semantic input key of a input field.
    fn get_semantic_input_key(
        &self,
        step_mode: VertexStepMode,
        format: VertexFormat,
        name: &str,
    ) -> Option<SemanticShaderInputKey>;
    /// Gets the semantic output key of a output item.
    fn get_semantic_output_key(&self, location: u32, name: &str)
        -> Option<SemanticShaderOutputKey>;
}
