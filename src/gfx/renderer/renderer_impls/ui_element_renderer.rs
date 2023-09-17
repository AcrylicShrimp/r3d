use crate::{
    gfx::{
        semantic_bindings,
        semantic_inputs::{self, KEY_POSITION},
        BindGroupLayoutCache, BindGroupProvider, CachedPipeline, Color, GenericBufferAllocation,
        HostBuffer, InstanceDataProvider, Material, MaterialHandle, NinePatchHandle, PipelineCache,
        PipelineProvider, Renderer, RendererVertexBufferAttribute, RendererVertexBufferLayout,
        SemanticShaderBindingKey, SemanticShaderInputKey, ShaderManager, SpriteHandle,
        TextureHandle, VertexBuffer, VertexBufferProvider,
    },
    ui::UISize,
};
use parking_lot::RwLockReadGuard;
use specs::{prelude::*, Component};
use std::{mem::size_of, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, Buffer, BufferAddress, BufferSize, BufferUsages, CompareFunction,
    DepthStencilState, Device, Face, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology,
    SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType, TextureViewDimension,
};
use zerocopy::AsBytes;

#[derive(Clone)]
pub enum UIElementSprite {
    Sprite(SpriteHandle),
    NinePatch(NinePatchHandle),
}

impl UIElementSprite {
    pub fn sprite(sprite: SpriteHandle) -> Self {
        Self::Sprite(sprite)
    }

    pub fn nine_patch(nine_patch: NinePatchHandle) -> Self {
        Self::NinePatch(nine_patch)
    }

    pub fn texture(&self) -> &TextureHandle {
        match self {
            UIElementSprite::Sprite(sprite) => sprite.texture(),
            UIElementSprite::NinePatch(nine_patch) => nine_patch.texture(),
        }
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct UIElementRenderer {
    mask: u32,
    color: Color,
    pipeline_provider: PipelineProvider,
    sprite: Option<UIElementSprite>,
    sprite_texture_bind_group: Option<Arc<BindGroup>>,
    sprite_sampler_bind_group: Option<Arc<BindGroup>>,
    vertex_buffer: Option<GenericBufferAllocation<Buffer>>,
}

impl UIElementRenderer {
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
            pipeline_provider,
            sprite: None,
            sprite_texture_bind_group: None,
            sprite_sampler_bind_group: None,
            vertex_buffer: None,
        }
    }

    pub fn mask(&self) -> u32 {
        self.mask
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_material(&mut self, material: MaterialHandle) {
        self.pipeline_provider.set_material(material);
    }

    pub fn set_sprite(
        &mut self,
        sprite: UIElementSprite,
        device: &Device,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
    ) {
        let sprite_texture_bind_group_layout =
            bind_group_layout_cache.create_layout(vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }]);
        let sprite_sampler_bind_group_layout =
            bind_group_layout_cache.create_layout(vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            }]);

        self.sprite_texture_bind_group =
            Some(Arc::new(device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: sprite_texture_bind_group_layout.as_ref(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&sprite.texture().view),
                }],
            })));
        self.sprite_sampler_bind_group =
            Some(Arc::new(device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: sprite_sampler_bind_group_layout.as_ref(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&sprite.texture().sampler),
                }],
            })));
        self.sprite = Some(sprite);

        // Since ui elements are always left-bottom based, positions must in range [0, 1].
        let vertices = vec![
            0.0f32, 0.0f32, 0.0f32, // bottom left position
            1.0f32, 0.0f32, 0.0f32, // bottom right position
            1.0f32, 1.0f32, 0.0f32, // top right position
            0.0f32, 0.0f32, 0.0f32, // bottom left position
            1.0f32, 1.0f32, 0.0f32, // top right position
            0.0f32, 1.0f32, 0.0f32, // top left position
        ];

        self.vertex_buffer = Some(GenericBufferAllocation::new(
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: vertices.as_bytes(),
                usage: BufferUsages::VERTEX,
            }),
            0,
            BufferSize::new((size_of::<f32>() * vertices.len()) as u64).unwrap(),
        ));
    }

    pub fn sub_renderer(
        &mut self,
        size: UISize,
        shader_mgr: &ShaderManager,
        pipeline_cache: &mut PipelineCache,
    ) -> Option<UIElementSubRenderer> {
        let pipeline = self
            .pipeline_provider
            .obtain_pipeline(shader_mgr, pipeline_cache)?;
        let material = self.pipeline_provider.material().cloned()?;
        let vertex_buffer = self.vertex_buffer.clone()?;
        let sprite = self.sprite.clone()?;
        let sprite_texture_bind_group = self.sprite_texture_bind_group.clone()?;
        let sprite_sampler_bind_group = self.sprite_sampler_bind_group.clone()?;

        Some(UIElementSubRenderer {
            pipeline,
            material,
            instance_count: match &sprite {
                UIElementSprite::Sprite(_) => 1,
                UIElementSprite::NinePatch(_) => 9,
            },
            bind_group_provider: UIElementRendererBindGroupProvider {
                sprite_texture_bind_group,
                sprite_sampler_bind_group,
            },
            vertex_buffer_provider: UIElementRendererVertexBufferProvider { vertex_buffer },
            instance_data_provider: UIElementRendererInstanceDataProvider {
                sprite,
                size,
                color: self.color,
            },
        })
    }
}

pub struct UIElementSubRenderer {
    pipeline: CachedPipeline,
    material: MaterialHandle,
    instance_count: u32,
    bind_group_provider: UIElementRendererBindGroupProvider,
    vertex_buffer_provider: UIElementRendererVertexBufferProvider,
    instance_data_provider: UIElementRendererInstanceDataProvider,
}

impl Renderer for UIElementSubRenderer {
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

    fn vertex_buffer_provider(&self) -> &dyn crate::gfx::VertexBufferProvider {
        &self.vertex_buffer_provider
    }

    fn instance_data_provider(&self) -> &dyn InstanceDataProvider {
        &self.instance_data_provider
    }
}

struct UIElementRendererBindGroupProvider {
    sprite_texture_bind_group: Arc<BindGroup>,
    sprite_sampler_bind_group: Arc<BindGroup>,
}

impl BindGroupProvider for UIElementRendererBindGroupProvider {
    fn bind_group(&self, _instance: u32, key: SemanticShaderBindingKey) -> Option<&BindGroup> {
        match key {
            semantic_bindings::KEY_SPRITE_TEXTURE => Some(&self.sprite_texture_bind_group),
            semantic_bindings::KEY_SPRITE_SAMPLER => Some(&self.sprite_sampler_bind_group),
            _ => None,
        }
    }
}

struct UIElementRendererVertexBufferProvider {
    vertex_buffer: GenericBufferAllocation<Buffer>,
}

impl VertexBufferProvider for UIElementRendererVertexBufferProvider {
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

struct UIElementRendererInstanceDataProvider {
    sprite: UIElementSprite,
    size: UISize,
    color: Color,
}

impl InstanceDataProvider for UIElementRendererInstanceDataProvider {
    fn copy_per_instance_data(
        &self,
        instance: u32,
        key: SemanticShaderInputKey,
        buffer: &mut GenericBufferAllocation<HostBuffer>,
    ) {
        match key {
            semantic_inputs::KEY_SPRITE_SIZE => {
                buffer.copy_from_slice(
                    [self.compute_size_x(instance), self.compute_size_y(instance)].as_bytes(),
                );
            }
            semantic_inputs::KEY_SPRITE_OFFSET => {
                buffer.copy_from_slice(
                    [
                        self.compute_offset_x(instance),
                        self.compute_offset_y(instance),
                    ]
                    .as_bytes(),
                );
            }
            semantic_inputs::KEY_SPRITE_UV_MIN => {
                let uv_min = match &self.sprite {
                    UIElementSprite::Sprite(sprite) => {
                        let mapping = sprite.mapping();
                        [
                            mapping.x_min as f32 / sprite.texture().width as f32,
                            mapping.y_min as f32 / sprite.texture().height as f32,
                        ]
                    }
                    UIElementSprite::NinePatch(nine_patch) => {
                        let x = match instance {
                            0 | 3 | 6 => nine_patch.mapping().x_min,
                            1 | 4 | 7 => nine_patch.mapping().x_mid_left,
                            2 | 5 | 8 => nine_patch.mapping().x_mid_right,
                            _ => return,
                        };
                        let y = match instance {
                            0 | 1 | 2 => nine_patch.mapping().y_mid_top,
                            3 | 4 | 5 => nine_patch.mapping().y_mid_bottom,
                            6 | 7 | 8 => nine_patch.mapping().y_min,
                            _ => return,
                        };
                        [
                            x as f32 / nine_patch.texture().width as f32,
                            y as f32 / nine_patch.texture().height as f32,
                        ]
                    }
                };
                buffer.copy_from_slice(uv_min.as_bytes());
            }
            semantic_inputs::KEY_SPRITE_UV_MAX => {
                let uv_min = match &self.sprite {
                    UIElementSprite::Sprite(sprite) => {
                        let mapping = sprite.mapping();
                        [
                            mapping.x_max as f32 / sprite.texture().width as f32,
                            mapping.y_max as f32 / sprite.texture().height as f32,
                        ]
                    }
                    UIElementSprite::NinePatch(nine_patch) => {
                        let x = match instance {
                            0 | 3 | 6 => nine_patch.mapping().x_mid_left,
                            1 | 4 | 7 => nine_patch.mapping().x_mid_right,
                            2 | 5 | 8 => nine_patch.mapping().x_max,
                            _ => return,
                        };
                        let y = match instance {
                            0 | 1 | 2 => nine_patch.mapping().y_max,
                            3 | 4 | 5 => nine_patch.mapping().y_mid_top,
                            6 | 7 | 8 => nine_patch.mapping().y_mid_bottom,
                            _ => return,
                        };
                        [
                            x as f32 / nine_patch.texture().width as f32,
                            y as f32 / nine_patch.texture().height as f32,
                        ]
                    }
                };
                buffer.copy_from_slice(uv_min.as_bytes());
            }
            semantic_inputs::KEY_SPRITE_COLOR => {
                buffer.copy_from_slice(
                    [self.color.r, self.color.g, self.color.b, self.color.a].as_bytes(),
                );
            }
            _ => {}
        }
    }
}

impl UIElementRendererInstanceDataProvider {
    fn compute_size_x(&self, instance: u32) -> f32 {
        let nine_patch = if let UIElementSprite::NinePatch(nine_patch) = &self.sprite {
            nine_patch
        } else {
            return self.size.width;
        };

        match instance {
            0 | 3 | 6 => {
                let mapping = nine_patch.mapping();
                let min_width = (mapping.width() - mapping.mid_width()) as f32;
                let ratio = f32::min(1.0, self.size.width / min_width);
                u16::abs_diff(mapping.x_min, mapping.x_mid_left) as f32 * ratio
            }
            1 | 4 | 7 => {
                let mapping = nine_patch.mapping();
                let min_width = mapping.width() - mapping.mid_width();
                f32::max(0.0, self.size.width - min_width as f32)
            }
            2 | 5 | 8 => {
                let mapping = nine_patch.mapping();
                let min_width = (mapping.width() - mapping.mid_width()) as f32;
                let ratio = f32::min(1.0, self.size.width / min_width);
                u16::abs_diff(mapping.x_mid_right, mapping.x_max) as f32 * ratio
            }
            _ => 0.0,
        }
    }

    fn compute_size_y(&self, instance: u32) -> f32 {
        let nine_patch = if let UIElementSprite::NinePatch(nine_patch) = &self.sprite {
            nine_patch
        } else {
            return self.size.height;
        };

        match instance {
            0 | 1 | 2 => {
                let mapping = nine_patch.mapping();
                let min_height = (mapping.height() - mapping.mid_height()) as f32;
                let ratio = f32::min(1.0, self.size.height / min_height);
                u16::abs_diff(mapping.y_mid_top, mapping.y_max) as f32 * ratio
            }
            3 | 4 | 5 => {
                let mapping = nine_patch.mapping();
                let min_height = mapping.height() - mapping.mid_height();
                f32::max(0.0, self.size.height - min_height as f32)
            }
            6 | 7 | 8 => {
                let mapping = nine_patch.mapping();
                let min_height = (mapping.height() - mapping.mid_height()) as f32;
                let ratio = f32::min(1.0, self.size.height / min_height);
                u16::abs_diff(mapping.y_min, mapping.y_mid_bottom) as f32 * ratio
            }
            _ => 0.0,
        }
    }

    fn compute_offset_x(&self, instance: u32) -> f32 {
        if let UIElementSprite::Sprite(_) = &self.sprite {
            return 0.0;
        }

        match instance {
            0 | 3 | 6 => 0f32,
            1 | 4 | 7 => self.compute_size_x(instance - 1),
            2 | 5 | 8 => self.size.width - self.compute_size_x(instance - 2),
            _ => 0.0,
        }
    }

    fn compute_offset_y(&self, instance: u32) -> f32 {
        if let UIElementSprite::Sprite(_) = &self.sprite {
            return 0.0;
        }

        match instance {
            0 | 1 | 2 => self.size.height - self.compute_size_y(instance),
            3 | 4 | 5 => self.compute_size_y(instance + 3),
            6 | 7 | 8 => 0.0,
            _ => 0.0,
        }
    }
}
