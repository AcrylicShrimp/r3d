use fontdue::layout::{HorizontalAlign, VerticalAlign, WrapStyle};

#[derive(Clone)]
pub struct GlyphLayoutConfig {
    pub horizontal_align: HorizontalAlign,
    pub vertical_align: VerticalAlign,
    pub wrap_style: WrapStyle,
    pub wrap_hard_breaks: bool,
}

impl GlyphLayoutConfig {
    pub fn new(
        horizontal_align: HorizontalAlign,
        vertical_align: VerticalAlign,
        wrap_style: WrapStyle,
        wrap_hard_breaks: bool,
    ) -> Self {
        Self {
            horizontal_align,
            vertical_align,
            wrap_style,
            wrap_hard_breaks,
        }
    }
}

impl Default for GlyphLayoutConfig {
    fn default() -> Self {
        Self {
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap_style: WrapStyle::Word,
            wrap_hard_breaks: true,
        }
    }
}
