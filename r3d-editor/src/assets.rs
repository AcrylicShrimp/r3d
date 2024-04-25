use lazy_static::lazy_static;
use r3d::{
    gfx::{Font, FontHandle, Material, MaterialHandle, ShaderHandle},
    use_context,
};
use std::path::Path;

lazy_static! {
    static ref SHADER_SPRITE: ShaderHandle = create_shader("r3d-editor/assets/shaders/sprite.wgsl");
    static ref SHADER_GLYPH: ShaderHandle = create_shader("r3d-editor/assets/shaders/glyph.wgsl");
}

lazy_static! {
    pub static ref MATERIAL_SPRITE: MaterialHandle = create_sprite_material();
    pub static ref MATERIAL_GLYPH: MaterialHandle = create_glyph_material();
}

lazy_static! {
    pub static ref FONT: FontHandle = create_font("r3d-editor/assets/fonts/NotoSans-Regular.ttf");
}

fn create_shader(path: impl AsRef<Path>) -> ShaderHandle {
    let source = std::fs::read_to_string(path).unwrap();
    let ctx = use_context();
    ctx.shader_mgr()
        .create_shader(ctx.render_mgr_mut().bind_group_layout_cache(), source)
        .unwrap()
}

fn create_font(path: impl AsRef<Path>) -> FontHandle {
    let font = std::fs::read(path).unwrap();
    FontHandle::new(Font::with_default(
        r3d::fontdue::Font::from_bytes(font, r3d::fontdue::FontSettings::default()).unwrap(),
    ))
}

pub fn create_sprite_material() -> MaterialHandle {
    let ctx = use_context();
    MaterialHandle::new(Material::new(
        SHADER_SPRITE.clone(),
        ctx.render_mgr_mut().pipeline_layout_cache(),
    ))
}

pub fn create_glyph_material() -> MaterialHandle {
    let ctx = use_context();
    MaterialHandle::new(Material::new(
        SHADER_GLYPH.clone(),
        ctx.render_mgr_mut().pipeline_layout_cache(),
    ))
}
