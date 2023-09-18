use crate::{
    gfx::{
        compute_glyph_layout, semantic_bindings,
        semantic_inputs::{self, KEY_POSITION},
        BindGroupLayoutCache, BindGroupProvider, CachedPipeline, Color, FontHandle,
        GenericBufferAllocation, GlyphLayoutConfig, GlyphManager, GlyphSpriteHandle, HostBuffer,
        InstanceDataProvider, Material, MaterialHandle, PipelineCache, PipelineProvider, Renderer,
        RendererVertexBufferAttribute, RendererVertexBufferLayout, SemanticShaderBindingKey,
        SemanticShaderInputKey, ShaderManager, VertexBuffer, VertexBufferProvider,
    },
    math::Vec2,
    ui::UISize,
};
use itertools::Itertools;
use parking_lot::RwLockReadGuard;
use specs::{prelude::*, Component};
use std::{mem::size_of, sync::Arc};
use wgpu::{
    BindGroup, Buffer, BufferAddress, CompareFunction, DepthStencilState, Face, FrontFace,
    PolygonMode, PrimitiveState, PrimitiveTopology, TextureFormat,
};
use zerocopy::AsBytes;

#[derive(Clone)]
struct Glyph {
    pub size: Vec2,
    pub offset: Vec2,
    pub sprite: GlyphSpriteHandle,
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
    layout_config: GlyphLayoutConfig,
    is_dirty: bool,
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
            layout_config: Default::default(),
            is_dirty: true,
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

    pub fn with_config<R>(&mut self, f: impl FnOnce(&mut GlyphLayoutConfig) -> R) -> R {
        let r = f(&mut self.layout_config);
        self.is_dirty = true;
        r
    }

    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
        self.is_dirty = true;
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

    pub fn set_font(&mut self, font: FontHandle) {
        self.font = Some(font);
        self.is_dirty = true;
    }

    pub fn set_text(&mut self, text: String) {
        self.text = Some(text);
        self.is_dirty = true;
    }

    pub fn sub_renderers<'a>(
        &'a mut self,
        is_dirty: bool,
        size: UISize,
        standard_ui_vertex_buffer: &GenericBufferAllocation<Buffer>,
        shader_mgr: &ShaderManager,
        glyph_mgr: &mut GlyphManager,
        pipeline_cache: &mut PipelineCache,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
    ) -> Option<Vec<UITextSubRenderer>> {
        self.update_glyphs(is_dirty, size, glyph_mgr, bind_group_layout_cache);

        let pipeline = self
            .pipeline_provider
            .obtain_pipeline(shader_mgr, pipeline_cache)?;
        let material = self.pipeline_provider.material().cloned()?;

        let groups = self
            .glyphs
            .iter()
            .group_by(|&glyph| Arc::as_ptr(glyph.sprite.texture_bind_group()));

        Some(Vec::from_iter(groups.into_iter().filter_map(
            |(_, group)| {
                let glyphs = Vec::from_iter(group.cloned());

                let first = if let Some(first) = glyphs.first() {
                    first
                } else {
                    return None;
                };
                let glyph_texture_bind_group = first.sprite.texture_bind_group().clone();
                let glyph_sampler_bind_group = first.sprite.sampler_bind_group().clone();

                Some(UITextSubRenderer {
                    pipeline: pipeline.clone(),
                    material: material.clone(),
                    instance_count: glyphs.len() as u32,
                    bind_group_provider: UITextRendererBindGroupProvider {
                        glyph_texture_bind_group,
                        glyph_sampler_bind_group,
                    },
                    vertex_buffer_provider: UITextRendererVertexBufferProvider {
                        vertex_buffer: standard_ui_vertex_buffer.clone(),
                    },
                    instance_data_provider: UITextRendererInstanceDataProvider {
                        glyphs,
                        color: self.color,
                        thickness: self.thickness,
                        smoothness: self.smoothness,
                    },
                })
            },
        )))
    }

    fn update_glyphs(
        &mut self,
        is_dirty: bool,
        size: UISize,
        glyph_mgr: &mut GlyphManager,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
    ) {
        if !self.is_dirty && !is_dirty {
            return;
        }

        let (font, text) = match (&self.font, &self.text) {
            (Some(font), Some(text)) => (font, text),
            _ => return,
        };

        self.glyphs.clear();

        for glyph in compute_glyph_layout(
            font,
            self.font_size,
            size,
            &self.layout_config,
            text.chars(),
        ) {
            self.glyphs.push(Glyph {
                size: glyph.size,
                offset: glyph.offset,
                sprite: glyph_mgr
                    .glyph(bind_group_layout_cache, font, glyph.key)
                    .clone(),
            });
        }

        self.glyphs
            .sort_unstable_by_key(|glyph| Arc::as_ptr(glyph.sprite.texture_bind_group()));
        self.is_dirty = false;
    }
}

pub struct UITextSubRenderer {
    pipeline: CachedPipeline,
    material: MaterialHandle,
    instance_count: u32,
    bind_group_provider: UITextRendererBindGroupProvider,
    vertex_buffer_provider: UITextRendererVertexBufferProvider,
    instance_data_provider: UITextRendererInstanceDataProvider,
}

impl Renderer for UITextSubRenderer {
    fn pipeline(&self) -> CachedPipeline {
        self.pipeline.clone()
    }

    fn material(&self) -> RwLockReadGuard<Material> {
        self.material.read()
    }

    fn instance_count(&self) -> u32 {
        self.instance_count
    }

    fn vertex_count(&self) -> u32 {
        6
    }

    fn bind_group_provider(&self) -> &dyn BindGroupProvider {
        &self.bind_group_provider
    }

    fn vertex_buffer_provider(&self) -> &dyn VertexBufferProvider {
        &self.vertex_buffer_provider
    }

    fn instance_data_provider(&self) -> &dyn InstanceDataProvider {
        &self.instance_data_provider
    }
}

struct UITextRendererBindGroupProvider {
    glyph_texture_bind_group: Arc<BindGroup>,
    glyph_sampler_bind_group: Arc<BindGroup>,
}

impl BindGroupProvider for UITextRendererBindGroupProvider {
    fn bind_group(&self, _instance: u32, key: SemanticShaderBindingKey) -> Option<&BindGroup> {
        match key {
            semantic_bindings::KEY_SPRITE_TEXTURE => Some(&self.glyph_texture_bind_group),
            semantic_bindings::KEY_SPRITE_SAMPLER => Some(&self.glyph_sampler_bind_group),
            _ => None,
        }
    }
}

struct UITextRendererVertexBufferProvider {
    vertex_buffer: GenericBufferAllocation<Buffer>,
}

impl VertexBufferProvider for UITextRendererVertexBufferProvider {
    fn vertex_buffer(&self, key: SemanticShaderInputKey) -> Option<VertexBuffer> {
        match key {
            semantic_inputs::KEY_POSITION => Some(VertexBuffer {
                slot: 0,
                buffer: &self.vertex_buffer,
            }),
            _ => None,
        }
    }
}

struct UITextRendererInstanceDataProvider {
    glyphs: Vec<Glyph>,
    color: Color,
    thickness: f32,
    smoothness: f32,
}

impl InstanceDataProvider for UITextRendererInstanceDataProvider {
    fn copy_per_instance_data(
        &self,
        instance: u32,
        key: SemanticShaderInputKey,
        buffer: &mut GenericBufferAllocation<HostBuffer>,
    ) {
        match key {
            semantic_inputs::KEY_SPRITE_SIZE => {
                let glyph = &self.glyphs[instance as usize];
                buffer.copy_from_slice([glyph.size.x, glyph.size.y].as_bytes());
            }
            semantic_inputs::KEY_SPRITE_OFFSET => {
                let glyph = &self.glyphs[instance as usize];
                buffer.copy_from_slice([glyph.offset.x, glyph.offset.y].as_bytes());
            }
            semantic_inputs::KEY_SPRITE_UV_MIN => {
                let glyph = &self.glyphs[instance as usize];
                let mapping = glyph.sprite.mapping();
                buffer.copy_from_slice(
                    [
                        mapping.x_min as f32 / glyph.sprite.texture().width as f32,
                        mapping.y_min as f32 / glyph.sprite.texture().height as f32,
                    ]
                    .as_bytes(),
                );
            }
            semantic_inputs::KEY_SPRITE_UV_MAX => {
                let glyph = &self.glyphs[instance as usize];
                let mapping = glyph.sprite.mapping();
                buffer.copy_from_slice(
                    [
                        mapping.x_max as f32 / glyph.sprite.texture().width as f32,
                        mapping.y_max as f32 / glyph.sprite.texture().height as f32,
                    ]
                    .as_bytes(),
                );
            }
            semantic_inputs::KEY_SPRITE_COLOR => {
                buffer.copy_from_slice(
                    [self.color.r, self.color.g, self.color.b, self.color.a].as_bytes(),
                );
            }
            semantic_inputs::KEY_GLYPH_THICKNESS => {
                buffer.copy_from_slice([self.thickness].as_bytes());
            }
            semantic_inputs::KEY_GLYPH_SMOOTHNESS => {
                buffer.copy_from_slice([self.smoothness].as_bytes());
            }
            _ => {}
        }
    }
}
