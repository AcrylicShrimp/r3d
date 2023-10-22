use crate::{AssetPipeline, PipelineGfxBridge};
use asset::assets::FontSource;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Default, Serialize, Deserialize)]
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

impl Default for FontTable {
    fn default() -> Self {
        Self {
            sdf_font_size: 32f32,
            sdf_inset: 8,
            sdf_radius: 3,
            sdf_cutoff: 0.25f32,
        }
    }
}

impl AssetPipeline for FontSource {
    type Metadata = FontMetadata;

    fn process(
        _file_path: &Path,
        file_content: Vec<u8>,
        metadata: &Self::Metadata,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            font_file: file_content,
            sdf_font_size: metadata.font.sdf_font_size,
            sdf_inset: metadata.font.sdf_inset,
            sdf_radius: metadata.font.sdf_radius,
            sdf_cutoff: metadata.font.sdf_cutoff,
        })
    }
}
