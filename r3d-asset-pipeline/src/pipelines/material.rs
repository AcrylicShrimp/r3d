use crate::{AssetPipeline, PipelineGfxBridge};
use anyhow::{anyhow, Context};
use asset::assets::MaterialSource;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Default, Serialize, Deserialize)]
pub struct MaterialMetadata;

impl AssetPipeline for MaterialSource {
    type Metadata = MaterialMetadata;

    fn process(
        _file_path: &Path,
        file_content: Vec<u8>,
        _metadata: &Self::Metadata,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        Self::deserialize(&file_content)
            .with_context(|| "failed to deserialize material")
            .map_err(|err| anyhow!(err))
    }
}
