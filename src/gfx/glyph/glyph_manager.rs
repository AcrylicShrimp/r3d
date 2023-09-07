use super::{generate_sdf, GlyphSprite, GlyphTexture};
use crate::gfx::{Font, FontHandle, GfxContextHandle};
use fontdue::layout::GlyphRasterConfig;
use std::collections::HashMap;

pub struct GlyphManager {
    gfx_ctx: GfxContextHandle,
    glyphs: HashMap<GlyphRasterConfig, GlyphSprite>,
    glyph_textures: HashMap<*const Font, Vec<GlyphTexture>>,
}

impl GlyphManager {
    pub fn new(gfx_ctx: GfxContextHandle) -> Self {
        Self {
            gfx_ctx,
            glyphs: HashMap::new(),
            glyph_textures: HashMap::new(),
        }
    }

    pub fn glyph(&mut self, font: &FontHandle, glyph: GlyphRasterConfig) -> &GlyphSprite {
        if !self.glyphs.contains_key(&glyph) {
            let (metrics, rasterized) = font
                .data
                .rasterize_indexed(glyph.glyph_index as _, font.sdf_font_size);
            let sdf = generate_sdf(
                &metrics,
                &rasterized,
                font.sdf_inset,
                font.sdf_radius,
                font.sdf_cutoff,
            );
            let glyph_textures = self
                .glyph_textures
                .entry(font.as_ptr())
                .or_insert_with(|| Vec::with_capacity(2));

            for glyph_texture in glyph_textures.iter_mut() {
                if let Some(mapping) = glyph_texture.glyph(
                    &self.gfx_ctx.queue,
                    (metrics.width + 2 * font.sdf_inset) as u16,
                    (metrics.height + 2 * font.sdf_inset) as u16,
                    &sdf,
                ) {
                    self.glyphs.insert(
                        glyph,
                        GlyphSprite::new(glyph_texture.texture().clone(), mapping),
                    );
                    return self.glyphs.get(&glyph).unwrap();
                }
            }

            let mut glyph_texture = GlyphTexture::new(&self.gfx_ctx.device, font.clone());
            let mapping = glyph_texture
                .glyph(
                    &self.gfx_ctx.queue,
                    (metrics.width + 2 * font.sdf_inset) as u16,
                    (metrics.height + 2 * font.sdf_inset) as u16,
                    &sdf,
                )
                .unwrap();
            self.glyphs.insert(
                glyph,
                GlyphSprite::new(glyph_texture.texture().clone(), mapping),
            );
            glyph_textures.push(glyph_texture);
        }

        self.glyphs.get(&glyph).unwrap()
    }
}
