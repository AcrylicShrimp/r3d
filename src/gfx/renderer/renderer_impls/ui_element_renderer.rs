use crate::{
    gfx::{
        semantic_bindings,
        semantic_inputs::{self, KEY_POSITION},
        BindGroupLayoutCache, BindGroupProvider, Color, GenericBufferAllocation, HostBuffer,
        MaterialHandle, NinePatchHandle, PerInstanceDataProvider, PipelineProvider, Renderer,
        RendererVertexBufferAttribute, RendererVertexBufferLayout, SemanticShaderBindingKey,
        SemanticShaderInputKey, SpriteHandle, TextureHandle,
    },
    math::Vec2,
};
use specs::{prelude::*, Component};
use std::{mem::size_of, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, Buffer, BufferAddress, BufferSize, BufferUsages, Device, Face, FrontFace,
    PolygonMode, PrimitiveState, PrimitiveTopology, SamplerBindingType, ShaderStages,
    TextureSampleType, TextureViewDimension,
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

    pub fn bind_group_provider(&self) -> impl BindGroupProvider {
        UIElementRendererBindGroupProvider {
            sprite_texture_bind_group: self.sprite_texture_bind_group.clone(),
            sprite_sampler_bind_group: self.sprite_sampler_bind_group.clone(),
        }
    }

    pub fn per_instance_data_provider(&self, size: Vec2) -> impl PerInstanceDataProvider {
        UIElementRendererPerInstanceDataProvider {
            sprite: self.sprite.clone(),
            size,
            color: self.color,
        }
    }
}

impl Renderer for UIElementRenderer {
    fn pipeline_provider(&mut self) -> &mut PipelineProvider {
        &mut self.pipeline_provider
    }

    fn instance_count(&self) -> u32 {
        match &self.sprite {
            Some(sprite) => match sprite {
                UIElementSprite::Sprite(_) => 1,
                UIElementSprite::NinePatch(_) => 9,
            },
            None => 0,
        }
    }

    fn vertex_count(&self) -> u32 {
        match &self.sprite {
            Some(_) => 6,
            None => 0,
        }
    }

    fn vertex_buffers(&self) -> Vec<GenericBufferAllocation<Buffer>> {
        match &self.vertex_buffer {
            Some(buffer) => vec![buffer.clone()],
            None => Vec::new(),
        }
    }
}

pub struct UIElementRendererBindGroupProvider {
    sprite_texture_bind_group: Option<Arc<BindGroup>>,
    sprite_sampler_bind_group: Option<Arc<BindGroup>>,
}

impl BindGroupProvider for UIElementRendererBindGroupProvider {
    fn bind_group(&self, _instance: u32, key: SemanticShaderBindingKey) -> Option<&BindGroup> {
        match key {
            semantic_bindings::KEY_SPRITE_TEXTURE => self
                .sprite_texture_bind_group
                .as_ref()
                .map(|bind_group| bind_group.as_ref()),
            semantic_bindings::KEY_SPRITE_SAMPLER => self
                .sprite_sampler_bind_group
                .as_ref()
                .map(|bind_group| bind_group.as_ref()),
            _ => None,
        }
    }
}

pub struct UIElementRendererPerInstanceDataProvider {
    sprite: Option<UIElementSprite>,
    size: Vec2,
    color: Color,
}

impl PerInstanceDataProvider for UIElementRendererPerInstanceDataProvider {
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
            semantic_inputs::KEY_SPRITE_COLOR => {
                buffer.copy_from_slice(
                    [self.color.r, self.color.g, self.color.b, self.color.a].as_bytes(),
                );
            }
            _ => {}
        }
    }
}

impl UIElementRendererPerInstanceDataProvider {
    fn compute_size_x(&self, instance: u32) -> f32 {
        let sprite = if let Some(sprite) = &self.sprite {
            sprite
        } else {
            return 0.0;
        };

        let nine_patch = if let UIElementSprite::NinePatch(nine_patch) = sprite {
            nine_patch
        } else {
            return self.size.x;
        };

        match instance {
            0 | 3 | 6 => {
                let mapping = nine_patch.mapping();
                let min_width = (mapping.width() - mapping.mid_width()) as f32;
                let ratio = f32::min(1.0, self.size.x / min_width);
                u16::abs_diff(mapping.x_min, mapping.x_mid_left) as f32 * ratio
            }
            1 | 4 | 7 => {
                let mapping = nine_patch.mapping();
                let min_width = mapping.width() - mapping.mid_width();
                f32::max(0.0, self.size.x - min_width as f32)
            }
            2 | 5 | 8 => {
                let mapping = nine_patch.mapping();
                let min_width = (mapping.width() - mapping.mid_width()) as f32;
                let ratio = f32::min(1.0, self.size.x / min_width);
                u16::abs_diff(mapping.x_mid_right, mapping.x_max) as f32 * ratio
            }
            _ => 0.0,
        }
    }

    fn compute_size_y(&self, instance: u32) -> f32 {
        let sprite = if let Some(sprite) = &self.sprite {
            sprite
        } else {
            return 0.0;
        };

        let nine_patch = if let UIElementSprite::NinePatch(nine_patch) = sprite {
            nine_patch
        } else {
            return self.size.y;
        };

        match instance {
            0 | 1 | 2 => {
                let mapping = nine_patch.mapping();
                let min_height = (mapping.height() - mapping.mid_height()) as f32;
                let ratio = f32::min(1.0, self.size.y / min_height);
                u16::abs_diff(mapping.y_mid_top, mapping.y_max) as f32 * ratio
            }
            3 | 4 | 5 => {
                let mapping = nine_patch.mapping();
                let min_height = mapping.height() - mapping.mid_height();
                f32::max(0.0, self.size.y - min_height as f32)
            }
            6 | 7 | 8 => {
                let mapping = nine_patch.mapping();
                let min_height = (mapping.height() - mapping.mid_height()) as f32;
                let ratio = f32::min(1.0, self.size.y / min_height);
                u16::abs_diff(mapping.y_min, mapping.y_mid_bottom) as f32 * ratio
            }
            _ => 0.0,
        }
    }

    fn compute_offset_x(&self, instance: u32) -> f32 {
        let sprite = if let Some(sprite) = &self.sprite {
            sprite
        } else {
            return 0.0;
        };

        if let UIElementSprite::Sprite(_) = sprite {
            return 0.0;
        }

        match instance {
            0 | 3 | 6 => 0f32,
            1 | 4 | 7 => self.compute_size_x(instance - 1),
            2 | 5 | 8 => self.size.x - self.compute_size_x(instance - 2),
            _ => 0.0,
        }
    }

    fn compute_offset_y(&self, instance: u32) -> f32 {
        let sprite = if let Some(sprite) = &self.sprite {
            sprite
        } else {
            return 0.0;
        };

        if let UIElementSprite::Sprite(_) = sprite {
            return 0.0;
        }

        match instance {
            0 | 1 | 2 => self.size.y - self.compute_size_y(instance),
            3 | 4 | 5 => self.compute_size_y(instance + 3),
            6 | 7 | 8 => 0.0,
            _ => 0.0,
        }
    }
}
