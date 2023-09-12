use crate::gfx::{
    semantic_inputs::{KEY_NORMAL, KEY_POSITION, KEY_UV},
    BindGroupProvider, GenericBufferAllocation, HostBuffer, MaterialHandle, MeshHandle,
    PerInstanceDataProvider, PipelineProvider, Renderer, RendererVertexBufferAttribute,
    RendererVertexBufferLayout, SemanticShaderBindingKey, SemanticShaderInputKey,
};
use specs::{prelude::*, Component};
use std::{mem::size_of, sync::Arc};
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

    pub fn bind_group_provider(&self) -> impl BindGroupProvider {
        MeshRendererBindGroupProvider
    }

    pub fn per_instance_data_provider(&self) -> impl PerInstanceDataProvider {
        MeshRendererPerInstanceDataProvider
    }
}

impl Renderer for MeshRenderer {
    fn pipeline_provider(&mut self) -> &mut PipelineProvider {
        &mut self.pipeline_provider
    }

    fn instance_count(&self) -> u32 {
        match &self.mesh {
            Some(_) => 1,
            None => 0,
        }
    }

    fn vertex_count(&self) -> u32 {
        match &self.mesh {
            Some(mesh) => mesh.data.faces.len() as u32 * 3,
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

pub struct MeshRendererBindGroupProvider;

impl BindGroupProvider for MeshRendererBindGroupProvider {
    fn bind_group(&self, _instance: u32, _key: SemanticShaderBindingKey) -> Option<&BindGroup> {
        None
    }
}

pub struct MeshRendererPerInstanceDataProvider;

impl PerInstanceDataProvider for MeshRendererPerInstanceDataProvider {
    fn copy_per_instance_data(
        &self,
        _instance: u32,
        _key: SemanticShaderInputKey,
        _buffer: &mut GenericBufferAllocation<HostBuffer>,
    ) {
    }
}
