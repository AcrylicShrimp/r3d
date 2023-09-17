use crate::gfx::{
    semantic_inputs::{self, KEY_NORMAL, KEY_POSITION, KEY_UV},
    BindGroupProvider, CachedPipeline, GenericBufferAllocation, HostBuffer, InstanceDataProvider,
    Material, MaterialHandle, MeshHandle, PipelineCache, PipelineProvider, Renderer,
    RendererVertexBufferAttribute, RendererVertexBufferLayout, SemanticShaderBindingKey,
    SemanticShaderInputKey, ShaderManager, VertexBuffer, VertexBufferProvider,
};
use parking_lot::RwLockReadGuard;
use specs::{prelude::*, Component};
use std::mem::size_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, BufferAddress, BufferSize, BufferUsages, CompareFunction, DepthStencilState,
    Device, Face, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology, TextureFormat,
};
use zerocopy::AsBytes;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct MeshRenderer {
    mask: u32,
    pipeline_provider: PipelineProvider,
    mesh: Option<MeshHandle>,
    vertex_buffer: Option<GenericBufferAllocation<Buffer>>,
}

impl MeshRenderer {
    pub fn new() -> Self {
        let mut pipeline_provider = PipelineProvider::new();

        pipeline_provider.set_buffer_layouts(vec![RendererVertexBufferLayout {
            array_stride: size_of::<[f32; 8]>() as BufferAddress,
            attributes: vec![
                RendererVertexBufferAttribute {
                    key: KEY_POSITION,
                    offset: 0,
                },
                RendererVertexBufferAttribute {
                    key: KEY_NORMAL,
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                },
                RendererVertexBufferAttribute {
                    key: KEY_UV,
                    offset: size_of::<[f32; 6]>() as BufferAddress,
                },
            ],
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
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }));

        Self {
            mask: 0xFFFF_FFFF,
            pipeline_provider,
            mesh: None,
            vertex_buffer: None,
        }
    }

    pub fn mask(&self) -> u32 {
        self.mask
    }

    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    pub fn set_material(&mut self, material: MaterialHandle) {
        self.pipeline_provider.set_material(material);
    }

    pub fn set_mesh(&mut self, mesh: MeshHandle, device: &Device) {
        if mesh.data.vertices.is_empty() {
            self.mesh = None;
            self.vertex_buffer = None;
            return;
        }

        self.mesh = Some(mesh.clone());

        let mut vertices = Vec::with_capacity(mesh.data.faces.len() * 3 * (3 + 3 + 2));
        let uvs = mesh.data.texture_coords[0].as_ref().unwrap();

        for face in &mesh.data.faces {
            for &face_index in &face.0 {
                let vertex = &mesh.data.vertices[face_index as usize];
                vertices.push(vertex.x);
                vertices.push(vertex.y);
                vertices.push(vertex.z);

                let normal = &mesh.data.normals[face_index as usize];
                vertices.push(normal.x);
                vertices.push(normal.y);
                vertices.push(normal.z);

                let uv = &uvs[face_index as usize];
                vertices.push(uv.x);
                vertices.push(uv.y);
            }
        }

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
        shader_mgr: &ShaderManager,
        pipeline_cache: &mut PipelineCache,
    ) -> Option<MeshSubRenderer> {
        let pipeline = self
            .pipeline_provider
            .obtain_pipeline(shader_mgr, pipeline_cache)?;
        let material = self.pipeline_provider.material().cloned()?;
        let vertex_buffer = self.vertex_buffer.clone()?;
        let mesh = self.mesh.as_ref()?;

        Some(MeshSubRenderer {
            pipeline,
            material,
            vertex_count: mesh.data.faces.len() as u32 * 3,
            bind_group_provider: MeshRendererBindGroupProvider,
            vertex_buffer_provider: MeshRendererVertexBufferProvider { vertex_buffer },
            instance_data_provider: MeshRendererInstanceDataProvider,
        })
    }
}

pub struct MeshSubRenderer {
    pipeline: CachedPipeline,
    material: MaterialHandle,
    vertex_count: u32,
    bind_group_provider: MeshRendererBindGroupProvider,
    vertex_buffer_provider: MeshRendererVertexBufferProvider,
    instance_data_provider: MeshRendererInstanceDataProvider,
}

impl Renderer for MeshSubRenderer {
    fn pipeline(&self) -> CachedPipeline {
        self.pipeline.clone()
    }

    fn material(&self) -> RwLockReadGuard<Material> {
        self.material.read()
    }

    fn instance_count(&self) -> u32 {
        1
    }

    fn vertex_count(&self) -> u32 {
        self.vertex_count
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

struct MeshRendererBindGroupProvider;

impl BindGroupProvider for MeshRendererBindGroupProvider {
    fn bind_group(&self, _instance: u32, _key: SemanticShaderBindingKey) -> Option<&BindGroup> {
        None
    }
}

struct MeshRendererVertexBufferProvider {
    vertex_buffer: GenericBufferAllocation<Buffer>,
}

impl VertexBufferProvider for MeshRendererVertexBufferProvider {
    fn vertex_buffer(&self, key: SemanticShaderInputKey) -> Option<VertexBuffer> {
        match key {
            semantic_inputs::KEY_POSITION
            | semantic_inputs::KEY_NORMAL
            | semantic_inputs::KEY_UV => Some(VertexBuffer {
                slot: 0,
                buffer: &self.vertex_buffer,
            }),
            _ => None,
        }
    }
}

struct MeshRendererInstanceDataProvider;

impl InstanceDataProvider for MeshRendererInstanceDataProvider {
    fn copy_per_instance_data(
        &self,
        _instance: u32,
        _key: SemanticShaderInputKey,
        _buffer: &mut GenericBufferAllocation<HostBuffer>,
    ) {
    }
}
