use crate::{AssetPipeline, Metadata, PipelineGfxBridge};
use anyhow::{anyhow, Context};
use asset::assets::MaterialSource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MaterialMetadata;

impl AssetPipeline for MaterialSource {
    type Metadata = MaterialMetadata;

    fn process(
        file_content: Vec<u8>,
        _metadata: &Metadata<Self::Metadata>,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        Self::deserialize(&file_content)
            .with_context(|| "failed to deserialize material")
            .map_err(|err| anyhow!(err))
    }
}
