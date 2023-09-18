use super::GlyphLayoutConfig;
use crate::{gfx::Font, math::Vec2, ui::UISize};
use fontdue::layout::{GlyphRasterConfig, HorizontalAlign, VerticalAlign};

pub struct GlyphLayoutElement {
    pub size: Vec2,
    pub offset: Vec2,
    pub key: GlyphRasterConfig,
}

// TODO: Add vertical align: baseline.
pub fn compute_glyph_layout(
    font: &Font,
    font_size: f32,
    size: UISize,
    config: &GlyphLayoutConfig,
    mut chars: impl Iterator<Item = char>,
) -> Vec<GlyphLayoutElement> {
    let pixel_ratio = font_size / font.sdf_font_size;
    let inset = pixel_ratio * font.sdf_inset as f32;

    let mut lines = Vec::with_capacity(4);

    loop {
        let line = compute_glyph_layout_line(font, font_size, inset, &mut chars);

        if line.elements.is_empty() {
            break;
        }

        lines.push(line);
    }

    let total_height = font_size * lines.len() as f32;
    let vertical_offset = match config.vertical_align {
        VerticalAlign::Top => size.height - total_height,
        VerticalAlign::Middle => (size.height - total_height) * 0.5,
        VerticalAlign::Bottom => 0f32,
    };
    let line_count = lines.len();

    for (index, line) in lines.iter_mut().enumerate() {
        let horizontal_offset = match config.horizontal_align {
            HorizontalAlign::Left => 0f32,
            HorizontalAlign::Center => (size.width - line.width) * 0.5,
            HorizontalAlign::Right => size.width - line.width,
        };

        let lines_below = line_count - index - 1;
        let vertical_offset = vertical_offset + font_size * lines_below as f32;

        for element in line.elements.iter_mut() {
            element.offset.x += horizontal_offset;
            element.offset.y += vertical_offset;
        }
    }

    lines.into_iter().flat_map(|line| line.elements).collect()
}

struct GlyphLineLayout {
    pub width: f32,
    pub elements: Vec<GlyphLayoutElement>,
}

fn compute_glyph_layout_line(
    font: &Font,
    font_size: f32,
    inset: f32,
    chars: &mut impl Iterator<Item = char>,
) -> GlyphLineLayout {
    let mut prev = None;
    let mut acc_width = 0.0f32;
    // let mut acc_height_min = 0.0f32;
    // let mut acc_height_max = 0.0f32;
    let mut acc_horizontal_offset = 0f32;
    let mut elements = Vec::new();

    for c in chars {
        if c == '\n' {
            break;
        }

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
        // acc_height_min = acc_height_min.min(metrics.ymin as f32);
        // acc_height_max = acc_height_max.max(metrics.ymin as f32 + metrics.height as f32);
        acc_horizontal_offset += kern + metrics.advance_width;

        prev = Some(c);
    }

    GlyphLineLayout {
        width: acc_width,
        elements,
    }
}
