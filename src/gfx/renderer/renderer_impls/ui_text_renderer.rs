use crate::gfx::{
    semantic_inputs::KEY_POSITION, Color, FontHandle, GenericBufferAllocation, GlyphLayoutConfig,
    GlyphManager, GlyphSprite, MaterialHandle, PipelineProvider, RendererVertexBufferAttribute,
    RendererVertexBufferLayout,
};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use specs::{prelude::*, Component};
use std::mem::size_of;
use wgpu::{
    Buffer, BufferAddress, CompareFunction, DepthStencilState, Face, FrontFace, PolygonMode,
    PrimitiveState, PrimitiveTopology, TextureFormat,
};

struct Glyph {
    pub sprite: GlyphSprite,
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct UITextRenderer {
    mask: u32,
    color: Color,
    font_size: f32,
    thickness: f32,
    smoothness: f32,
    pipeline_provider: PipelineProvider,
    font: Option<FontHandle>,
    text: Option<String>,
    glyphs: Vec<Glyph>,
    layout: Layout,
    layout_config: GlyphLayoutConfig,
    vertex_buffer: Option<GenericBufferAllocation<Buffer>>,
}

impl UITextRenderer {
    pub fn new() -> Self {
        let mut pipeline_provider = PipelineProvider::new();

        pipeline_provider.set_buffer_layouts(vec![RendererVertexBufferLayout {
            array_stride: size_of::<[f32; 3]>() as BufferAddress,
            attributes: vec![RendererVertexBufferAttribute {
                key: KEY_POSITION,
                offset: 0,
            }],
        }]);
        pipeline_provider.set_primitive(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        });
        pipeline_provider.set_depth_stencil(Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil: Default::default(),
            bias: Default::default(),
        }));

        Self {
            mask: 0xFFFF_FFFF,
            color: Color::white(),
            font_size: 16f32,
            thickness: 0.5f32,
            smoothness: 0.125f32,
            pipeline_provider,
            font: None,
            text: None,
            glyphs: Vec::new(),
            layout: Layout::new(CoordinateSystem::PositiveYUp),
            layout_config: Default::default(),
            vertex_buffer: None,
        }
    }

    pub fn mask(&self) -> u32 {
        self.mask
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    pub fn smoothness(&self) -> f32 {
        self.smoothness
    }

    pub fn font(&self) -> Option<&FontHandle> {
        self.font.as_ref()
    }

    pub fn text(&self) -> Option<&String> {
        self.text.as_ref()
    }

    pub fn config(&self) -> &GlyphLayoutConfig {
        &self.layout_config
    }

    pub fn with_config<R>(
        &mut self,
        glyph_mgr: &mut GlyphManager,
        f: impl FnOnce(&mut GlyphLayoutConfig) -> R,
    ) -> R {
        let r = f(&mut self.layout_config);
        self.update_glyphs(glyph_mgr);
        r
    }

    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_font_size(&mut self, glyph_mgr: &mut GlyphManager, font_size: f32) {
        self.font_size = font_size;
        self.update_glyphs(glyph_mgr);
    }

    pub fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
    }

    pub fn set_smoothness(&mut self, smoothness: f32) {
        self.smoothness = smoothness;
    }

    pub fn set_material(&mut self, material: MaterialHandle) {
        self.pipeline_provider.set_material(material);
    }

    pub fn set_font(&mut self, glyph_mgr: &mut GlyphManager, font: FontHandle) {
        self.font = Some(font);
        self.update_glyphs(glyph_mgr);
    }

    pub fn set_text(&mut self, glyph_mgr: &mut GlyphManager, text: String) {
        self.text = Some(text);
        self.update_glyphs(glyph_mgr);
    }

    fn update_glyphs(&mut self, glyph_mgr: &mut GlyphManager) {
        let (font, text) = match (&self.font, &self.text) {
            (Some(font), Some(text)) => (font, text),
            _ => return,
        };

        self.layout.reset(&LayoutSettings {
            x: 0f32,
            y: 0f32,
            max_width: None,
            max_height: None,
            horizontal_align: self.layout_config.horizontal_align,
            vertical_align: self.layout_config.vertical_align,
            line_height: 1f32,
            wrap_style: self.layout_config.wrap_style,
            wrap_hard_breaks: self.layout_config.wrap_hard_breaks,
        });
        self.layout.append(
            &[&font.data],
            &TextStyle::new(text.as_ref(), self.font_size, 0),
        );

        self.glyphs.clear();

        for glyph in self.layout.glyphs() {
            self.glyphs.push(Glyph {
                sprite: glyph_mgr.glyph(font, glyph.key).clone(),
            });
        }
    }
}
