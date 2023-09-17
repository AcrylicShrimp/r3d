use super::{generate_sdf, GlyphSprite, GlyphSpriteHandle, GlyphTexture};
use crate::{
    gfx::{Font, FontHandle, GfxContextHandle},
    use_context,
};
use fontdue::layout::GlyphRasterConfig;
use std::collections::HashMap;

pub struct GlyphManager {
    gfx_ctx: GfxContextHandle,
    glyphs: HashMap<GlyphRasterConfig, GlyphSpriteHandle>,
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

    pub fn glyph(&mut self, font: &FontHandle, glyph: GlyphRasterConfig) -> GlyphSpriteHandle {
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
                        GlyphSpriteHandle::new(GlyphSprite::new(
                            glyph_texture.texture_bind_group().clone(),
                            glyph_texture.sampler_bind_group().clone(),
                            glyph_texture.texture().clone(),
                            mapping,
                        )),
                    );
                    return self.glyphs.get(&glyph).unwrap().clone();
                }
            }

            let ctx = use_context();
            let mut glyph_texture = GlyphTexture::new(
                &ctx.gfx_ctx.device,
                ctx.render_mgr_mut().bind_group_layout_cache(),
                font.clone(),
            );
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
                GlyphSpriteHandle::new(GlyphSprite::new(
                    glyph_texture.texture_bind_group().clone(),
                    glyph_texture.sampler_bind_group().clone(),
                    glyph_texture.texture().clone(),
                    mapping,
                )),
            );
            glyph_textures.push(glyph_texture);
        }

        self.glyphs.get(&glyph).unwrap().clone()
    }
}
