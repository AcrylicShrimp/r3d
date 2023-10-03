use crate::{Metadata, PipelineGfxBridge};
use asset::AssetSource;
use serde::{Deserialize, Serialize};

/// Represents a pipeline that can be used to process an asset.
pub trait AssetPipeline: Serialize + AssetSource
where
    Self: Sized,
{
    type Metadata: for<'de> Deserialize<'de>;

    /// Process the file content and metadata into a new asset source.
    fn process(
        file_content: Vec<u8>,
        metadata: &Metadata<Self::Metadata>,
        gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self>;
}