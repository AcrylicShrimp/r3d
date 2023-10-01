use crate::{AssetPipeline, Metadata, PipelineGfxBridge};
use anyhow::{anyhow, Context};
use asset::assets::{
    Mesh, MeshAABB, ModelSource, Node, NodeTransform, VertexAttribute, VertexAttributeKind,
    VertexIndexType,
};
use byteorder::ByteOrder;
use russimp::{
    mesh::PrimitiveType,
    scene::{PostProcess, Scene},
    Color4D, Vector3D,
};
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Serialize, Deserialize)]
pub struct MeshMetadata {
    pub mesh: MeshTable,
}

#[derive(Serialize, Deserialize)]
pub struct MeshTable {}

impl AssetPipeline for ModelSource {
    type Metadata = MeshMetadata;

    fn process(
        file_content: Vec<u8>,
        _metadata: &Metadata<Self::Metadata>,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        let scene = Scene::from_buffer(
            &file_content,
            vec![
                PostProcess::JoinIdenticalVertices,
                PostProcess::Triangulate,
                PostProcess::SortByPrimitiveType,
                PostProcess::SplitLargeMeshes,
                PostProcess::GenerateNormals,
                PostProcess::FixInfacingNormals,
                PostProcess::CalculateTangentSpace,
                PostProcess::GenerateUVCoords,
                PostProcess::GenerateBoundingBoxes,
                PostProcess::ImproveCacheLocality,
                PostProcess::OptimizeGraph,
                PostProcess::OptimizeMeshes,
            ],
            "",
        )
        .with_context(|| "failed to load mesh from file")
        .map_err(|err| anyhow!(err))?;
        let mut extractor = SceneExtractor::new();

        let root_node_index = scene
            .root
            .as_ref()
            .map(|root| extractor.extract_node(&scene, root, None));
        let nodes = extractor.nodes;
        let meshes = extractor.meshes;

        Ok(Self {
            root_node_index,
            nodes,
            meshes,
        })
    }
}

#[derive(Default)]
struct SceneExtractor {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Mesh>,
}

impl SceneExtractor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extract_node(
        &mut self,
        scene: &Scene,
        node: &russimp::node::Node,
        parent_index: Option<u32>,
    ) -> u32 {
        let index = self.nodes.len() as u32;
        self.nodes.push(Node {
            index,
            parent_index,
            children_indices: vec![],
            name: node.name.clone(),
            transform: NodeTransform {
                matrix: [
                    node.transformation.a1,
                    node.transformation.b1,
                    node.transformation.c1,
                    node.transformation.d1,
                    node.transformation.a2,
                    node.transformation.b2,
                    node.transformation.c2,
                    node.transformation.d2,
                    node.transformation.a3,
                    node.transformation.b3,
                    node.transformation.c3,
                    node.transformation.d3,
                    node.transformation.a4,
                    node.transformation.b4,
                    node.transformation.c4,
                    node.transformation.d4,
                ],
            },
            mesh_indices: vec![],
        });

        let children_indices = Vec::from_iter(
            node.children
                .borrow()
                .iter()
                .map(|child| self.extract_node(scene, child, Some(index))),
        );
        self.nodes[index as usize].children_indices = children_indices;

        let mesh_indices = Vec::from_iter(
            node.meshes
                .iter()
                .filter(|&index| {
                    scene.meshes[*index as usize].primitive_types == PrimitiveType::Triangle as u32
                })
                .map(|index| self.extract_mesh(&scene.meshes[*index as usize])),
        );
        self.nodes[index as usize].mesh_indices = mesh_indices;

        index as u32
    }

    fn extract_mesh(&mut self, mesh: &russimp::mesh::Mesh) -> u32 {
        let index = self.meshes.len() as u32;
        let mesh = convert_mesh(index, mesh);
        self.meshes.push(mesh);
        index
    }
}

fn convert_mesh(index: u32, mesh: &russimp::mesh::Mesh) -> Mesh {
    let mut vertex_attributes = Vec::with_capacity(8);
    let mut offset = 0;

    // Position
    vertex_attributes.push(VertexAttribute {
        offset,
        kind: VertexAttributeKind::Position,
    });
    offset += size_of::<[f32; 3]>() as u32;

    // Normal
    if !mesh.normals.is_empty() {
        vertex_attributes.push(VertexAttribute {
            offset,
            kind: VertexAttributeKind::Normal,
        });
    }
    offset += size_of::<[f32; 3]>() as u32;

    // Colors
    for (index, colors) in mesh.colors.iter().enumerate() {
        if colors.as_ref().is_some_and(|colors| !colors.is_empty()) {
            vertex_attributes.push(VertexAttribute {
                offset,
                kind: VertexAttributeKind::Color {
                    index: index as u32,
                },
            });
            offset += size_of::<[f32; 4]>() as u32;
        }
    }

    // Texture coordinates
    for (index, texture_coords) in mesh.texture_coords.iter().enumerate() {
        if texture_coords
            .as_ref()
            .is_some_and(|texture_coords| !texture_coords.is_empty())
        {
            vertex_attributes.push(VertexAttribute {
                offset,
                kind: VertexAttributeKind::TexCoord {
                    index: index as u32,
                },
            });
            offset += size_of::<[f32; 2]>() as u32;
        }
    }

    // Tangent
    if !mesh.tangents.is_empty() {
        vertex_attributes.push(VertexAttribute {
            offset,
            kind: VertexAttributeKind::Tangent,
        });
        offset += size_of::<[f32; 3]>() as u32;
    }

    // Bitangent
    if !mesh.bitangents.is_empty() {
        vertex_attributes.push(VertexAttribute {
            offset,
            kind: VertexAttributeKind::Bitangent,
        });
        offset += size_of::<[f32; 3]>() as u32;
    }

    let stride = (offset / size_of::<f32>() as u32) as usize;
    let mut vertex_buffer = vec![0f32; mesh.vertices.len() * stride];

    for attribute in &vertex_attributes {
        let source = match attribute.kind {
            VertexAttributeKind::Position => VertexDataCopySource::Vector3D(&mesh.vertices),
            VertexAttributeKind::Normal => VertexDataCopySource::Vector3D(&mesh.normals),
            VertexAttributeKind::Color { index } => {
                VertexDataCopySource::Color4D(mesh.colors[index as usize].as_ref().unwrap())
            }
            VertexAttributeKind::TexCoord { index } => VertexDataCopySource::Vector2D(
                mesh.texture_coords[index as usize].as_ref().unwrap(),
            ),
            VertexAttributeKind::Tangent => VertexDataCopySource::Vector3D(&mesh.tangents),
            VertexAttributeKind::Bitangent => VertexDataCopySource::Vector3D(&mesh.bitangents),
        };

        for index in 0..mesh.vertices.len() {
            source.copy_into(
                index,
                &mut vertex_buffer[index * stride + attribute.offset as usize / size_of::<f32>()..],
            );
        }
    }

    let mut raw_vertex_buffer = vec![0u8; vertex_buffer.len() * size_of::<f32>()];
    byteorder::LE::write_f32_into(&vertex_buffer, &mut raw_vertex_buffer);
    drop(vertex_buffer);

    let vertex_count = mesh.vertices.len();
    let (index_type, raw_index_buffer) = if vertex_count < u8::MAX as usize {
        let mut index_buffer = Vec::with_capacity(mesh.faces.len() * 3);

        for face in &mesh.faces {
            debug_assert_eq!(face.0.len(), 3);
            index_buffer.push(face.0[0] as u8);
            index_buffer.push(face.0[1] as u8);
            index_buffer.push(face.0[2] as u8);
        }

        (VertexIndexType::U8, index_buffer)
    } else if vertex_count < u16::MAX as usize {
        let mut index_buffer = Vec::with_capacity(mesh.faces.len() * 3);

        for face in &mesh.faces {
            debug_assert_eq!(face.0.len(), 3);
            index_buffer.push(face.0[0] as u16);
            index_buffer.push(face.0[1] as u16);
            index_buffer.push(face.0[2] as u16);
        }

        let mut raw_index_buffer = vec![0u8; index_buffer.len() * size_of::<u16>()];
        byteorder::LE::write_u16_into(&index_buffer, &mut raw_index_buffer);

        (VertexIndexType::U16, raw_index_buffer)
    } else {
        let mut index_buffer = Vec::with_capacity(mesh.faces.len() * 3);

        for face in &mesh.faces {
            debug_assert_eq!(face.0.len(), 3);
            index_buffer.push(face.0[0] as u32);
            index_buffer.push(face.0[1] as u32);
            index_buffer.push(face.0[2] as u32);
        }

        let mut raw_index_buffer = vec![0u8; index_buffer.len() * size_of::<u32>()];
        byteorder::LE::write_u32_into(&index_buffer, &mut raw_index_buffer);

        (VertexIndexType::U32, raw_index_buffer)
    };

    let aabb = MeshAABB {
        min: [mesh.aabb.min.x, mesh.aabb.min.y, mesh.aabb.min.z],
        max: [mesh.aabb.max.x, mesh.aabb.max.y, mesh.aabb.max.z],
    };

    Mesh {
        index,
        aabb,
        index_type,
        index_buffer: raw_index_buffer,
        vertex_attributes,
        vertex_buffer: raw_vertex_buffer,
        vertex_count: vertex_count as u32,
    }
}

#[derive(Clone, Copy)]
enum VertexDataCopySource<'a> {
    Vector2D(&'a [Vector3D]),
    Vector3D(&'a [Vector3D]),
    Color4D(&'a [Color4D]),
}

impl<'a> VertexDataCopySource<'a> {
    pub fn copy_into(&self, index: usize, dst: &mut [f32]) {
        match self {
            &VertexDataCopySource::Vector2D(src) => {
                let src = src[index];
                dst[0] = src.x;
                dst[1] = src.y;
            }
            &VertexDataCopySource::Vector3D(src) => {
                let src = src[index];
                dst[0] = src.x;
                dst[1] = src.y;
                dst[2] = src.z;
            }
            &VertexDataCopySource::Color4D(src) => {
                let src = src[index];
                dst[0] = src.r;
                dst[1] = src.g;
                dst[2] = src.b;
                dst[3] = src.a;
            }
        }
    }
}
