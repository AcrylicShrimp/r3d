use codegen::Handle;
use fontdue::Font as FontDueFont;

#[derive(Handle)]
pub struct Font {
    pub data: FontDueFont,
    pub sdf_font_size: f32,
    pub sdf_inset: usize,
    pub sdf_radius: usize,
    pub sdf_cutoff: f32,
}

impl Font {
    pub fn with_default(data: FontDueFont) -> Self {
        Self {
            data,
            sdf_font_size: 128f32,
            sdf_inset: 12usize,
            sdf_radius: 12usize,
            sdf_cutoff: 0.45f32,
        }
    }
}
