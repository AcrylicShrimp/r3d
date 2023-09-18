use super::GlyphLayoutConfig;
use crate::{gfx::Font, math::Vec2, ui::UISize};
use fontdue::layout::{GlyphRasterConfig, HorizontalAlign, VerticalAlign};

pub struct GlyphLayoutElement {
    pub size: Vec2,
    pub offset: Vec2,
    pub key: GlyphRasterConfig,
}

pub fn compute_glyph_layout(
    font: &Font,
    font_size: f32,
    size: UISize,
    config: &GlyphLayoutConfig,
    chars: impl Iterator<Item = char>,
) -> Vec<GlyphLayoutElement> {
    let pixel_ratio = font_size / font.sdf_font_size;
    let inset = pixel_ratio * font.sdf_inset as f32;

    let mut prev = None;
    let mut acc_width = 0.0f32;
    let mut acc_height_min = 0.0f32;
    let mut acc_height_max = 0.0f32;
    let mut acc_horizontal_offset = 0f32;

    let mut elements = Vec::new();

    for c in chars {
        let metrics = font.data.metrics(c, font_size);
        let kern = prev
            .and_then(|prev| font.data.horizontal_kern(prev, c, font_size))
            .unwrap_or(0.0f32);

        let offset = Vec2::new(
            -inset + metrics.xmin as f32 + kern + acc_horizontal_offset,
            -inset + metrics.ymin as f32,
        );
        let size = Vec2::new(
            metrics.width as f32 + inset * 2f32,
            metrics.height as f32 + inset * 2f32,
        );
        elements.push(GlyphLayoutElement {
            size,
            offset,
            key: GlyphRasterConfig {
                glyph_index: font.data.lookup_glyph_index(c),
                px: font_size,
                font_hash: font.data.file_hash(),
            },
        });

        acc_width += kern + metrics.advance_width;
        acc_height_min = acc_height_min.min(metrics.ymin as f32);
        acc_height_max = acc_height_max.max(metrics.ymin as f32 + metrics.height as f32);
        acc_horizontal_offset += kern + metrics.advance_width;

        prev = Some(c);
    }

    let horizontal_offset = match config.horizontal_align {
        HorizontalAlign::Left => 0f32,
        HorizontalAlign::Center => (size.width - acc_width) * 0.5,
        HorizontalAlign::Right => size.width - acc_width,
    };
    // TODO: Add vertical align: baseline.
    let vertical_offset = match config.vertical_align {
        VerticalAlign::Top => size.height - (acc_height_max - acc_height_min),
        VerticalAlign::Middle => (size.height - (acc_height_max - acc_height_min)) * 0.5,
        VerticalAlign::Bottom => -acc_height_min,
    };

    for element in elements.iter_mut() {
        element.offset.x += horizontal_offset;
        element.offset.y += vertical_offset;
    }

    elements
}
