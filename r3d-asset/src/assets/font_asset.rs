use crate::{Asset, AssetDepsProvider, AssetLoadError, AssetSource, GfxBridge, TypedAsset};
use fontdue::{Font as FontDueFont, Metrics as FontDueMetrics};
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    num::NonZeroU16,
    sync::Arc,
};
use uuid::Uuid;

/// Represents a single glyph to be identified. It is safe to share across all fonts.
#[derive(Debug, Clone, Copy)]
pub struct GlyphId {
    pub glyph_index: NonZeroU16,
    pub px: f32,
    pub font_hash: usize,
}

impl Hash for GlyphId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.glyph_index.hash(state);
        self.px.to_bits().hash(state);
        self.font_hash.hash(state);
    }
}

impl PartialEq for GlyphId {
    fn eq(&self, other: &Self) -> bool {
        self.glyph_index == other.glyph_index
            && self.px == other.px
            && self.font_hash == other.font_hash
    }
}

impl Eq for GlyphId {}

#[derive(Debug, Clone)]
pub struct GlyphMetrics {
    pub xmin: i32,
    pub ymin: i32,
    pub width: usize,
    pub height: usize,
    pub advance_width: f32,
    pub advance_height: f32,
}

impl From<FontDueMetrics> for GlyphMetrics {
    fn from(value: FontDueMetrics) -> Self {
        Self {
            xmin: value.xmin,
            ymin: value.ymin,
            width: value.width,
            height: value.height,
            advance_width: value.advance_width,
            advance_height: value.advance_height,
        }
    }
}

/// Represents a font asset. It supplies SDF generation parameters.
/// It also provides glyph metrics and rasterization.
pub trait FontAsset: Asset {
    fn sdf_font_size(&self) -> f32;
    fn sdf_inset(&self) -> u32;
    fn sdf_radius(&self) -> u32;
    fn sdf_cutoff(&self) -> f32;
    fn glyph_id(&self, character: char) -> Option<GlyphId>;
    fn glyph_metrics(&self, glyph_index: NonZeroU16) -> GlyphMetrics;
    fn rasterize(&self, glyph_index: NonZeroU16) -> Vec<u8>;
}

#[derive(Serialize, Deserialize)]
pub struct FontSource {
    pub font_file: Vec<u8>,
    pub sdf_font_size: f32,
    pub sdf_inset: u32,
    pub sdf_radius: u32,
    pub sdf_cutoff: f32,
}

impl AssetSource for FontSource {
    type Asset = dyn FontAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }

    fn load(
        self,
        id: Uuid,
        _deps_provider: &dyn AssetDepsProvider,
        _gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        Ok(Arc::new(Font {
            id,
            font: FontDueFont::from_bytes(self.font_file.as_ref(), Default::default())
                .map_err(|err| AssetLoadError::Other(err.to_owned()))?,
            sdf_font_size: self.sdf_font_size,
            sdf_inset: self.sdf_inset,
            sdf_radius: self.sdf_radius,
            sdf_cutoff: self.sdf_cutoff,
        }))
    }
}

struct Font {
    id: Uuid,
    font: FontDueFont,
    sdf_font_size: f32,
    sdf_inset: u32,
    sdf_radius: u32,
    sdf_cutoff: f32,
}

impl Asset for Font {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Font(self)
    }
}

impl FontAsset for Font {
    fn sdf_font_size(&self) -> f32 {
        self.sdf_font_size
    }

    fn sdf_inset(&self) -> u32 {
        self.sdf_inset
    }

    fn sdf_radius(&self) -> u32 {
        self.sdf_radius
    }

    fn sdf_cutoff(&self) -> f32 {
        self.sdf_cutoff
    }

    fn glyph_id(&self, character: char) -> Option<GlyphId> {
        Some(GlyphId {
            glyph_index: NonZeroU16::new(self.font.lookup_glyph_index(character))?,
            px: self.sdf_font_size,
            font_hash: self.font.file_hash(),
        })
    }

    fn glyph_metrics(&self, glyph_index: NonZeroU16) -> GlyphMetrics {
        self.font
            .metrics_indexed(glyph_index.get(), self.sdf_font_size)
            .into()
    }

    fn rasterize(&self, glyph_index: NonZeroU16) -> Vec<u8> {
        let (_, rasterized) = self
            .font
            .rasterize_indexed(glyph_index.get(), self.sdf_font_size);
        rasterized
    }
}
