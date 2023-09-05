use crate::engine::gfx::{
    semantic_inputs::{KEY_NORMAL, KEY_POSITION},
    GenericBufferAllocation, HostBuffer, MaterialHandle, MeshHandle, PipelineProvider, Renderer,
    RendererVertexBufferAttribute, RendererVertexBufferLayout, SemanticShaderInputKey,
};
use specs::{prelude::*, Component};
use std::mem::size_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferAddress, BufferSize, BufferUsages, CompareFunction, DepthStencilState, Device,
    Face, FrontFace, PolygonMode, PrimitiveState, PrimitiveTopology, TextureFormat,
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
            array_stride: size_of::<[f32; 6]>() as BufferAddress,
            attributes: vec![
                RendererVertexBufferAttribute {
                    key: KEY_POSITION,
                    offset: 0,
                },
                RendererVertexBufferAttribute {
                    key: KEY_NORMAL,
                    offset: size_of::<[f32; 3]>() as BufferAddress,
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

        let mut vertices = Vec::with_capacity(mesh.data.faces.len() * 3 * 2);

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
            }
        }

        self.vertex_buffer = Some(GenericBufferAllocation::new(
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: vertices.as_bytes(),
                usage: BufferUsages::VERTEX,
            }),
            0,
            BufferSize::new((mesh.data.faces.len() * 3 * 2 * size_of::<[f32; 3]>()) as u64)
                .unwrap(),
        ));
    }
}

impl Renderer for MeshRenderer {
    fn pipeline_provider(&mut self) -> &mut PipelineProvider {
        &mut self.pipeline_provider
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

    fn copy_semantic_per_instance_input(
        &self,
        _key: SemanticShaderInputKey,
        _allocation: &mut GenericBufferAllocation<HostBuffer>,
    ) {
    }
}
