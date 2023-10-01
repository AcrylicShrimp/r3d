use crate::{AssetPipeline, Metadata, PipelineGfxBridge};
use asset::assets::FontSource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FontMetadata {
    pub font: FontTable,
}

#[derive(Serialize, Deserialize)]
pub struct FontTable {
    pub sdf_font_size: f32,
    pub sdf_inset: u32,
    pub sdf_radius: u32,
    pub sdf_cutoff: f32,
}

impl AssetPipeline for FontSource {
    type Metadata = FontMetadata;

    fn process(
        file_content: Vec<u8>,
        metadata: &Metadata<Self::Metadata>,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            font_file: file_content,
            sdf_font_size: metadata.extra.font.sdf_font_size,
            sdf_inset: metadata.extra.font.sdf_inset,
            sdf_radius: metadata.extra.font.sdf_radius,
            sdf_cutoff: metadata.extra.font.sdf_cutoff,
        })
    }
}
