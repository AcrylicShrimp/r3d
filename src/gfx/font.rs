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
