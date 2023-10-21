use crate::{AssetPipeline, Metadata, PipelineGfxBridge};
use anyhow::{anyhow, Context};
use asset::assets::{
    MeshAABB, MeshSource, ModelSource, NodeSource, NodeTransform, VertexAttribute,
    VertexAttributeKind, VertexIndexType,
};
use byteorder::ByteOrder;
use pmx::Pmx;
use russimp::{
    mesh::PrimitiveType,
    scene::{PostProcess, Scene},
    Color4D, Vector3D,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    mem::size_of,
    path::Path,
};
use zerocopy::AsBytes;

#[derive(Serialize, Deserialize)]
pub struct MeshMetadata {
    pub mesh: MeshTable,
}

#[derive(Serialize, Deserialize)]
pub struct MeshTable {}

impl AssetPipeline for ModelSource {
    type Metadata = MeshMetadata;

    fn process(
        file_path: &Path,
        file_content: Vec<u8>,
        _metadata: &Metadata<Self::Metadata>,
        _gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        if file_path
            .extension()
            .map_or(false, |ext| ext.to_ascii_lowercase() == "pmx")
        {
            process_pmx_model(&file_content)
        } else {
            process_assimp_model(&file_content)
        }
    }
}

fn process_pmx_model(content: &[u8]) -> anyhow::Result<ModelSource> {
    let pmx = Pmx::parse(content).with_context(|| "failed to load mesh from file")?;

    let mut material_offset = 0;
    let mut meshes = Vec::with_capacity(pmx.materials.len());

    for material_index in 0..pmx.materials.len() {
        let material = &pmx.materials[material_index];
        let surfaces = &pmx.surfaces[material_offset..material.surface_count as usize];
        material_offset += material.surface_count as usize;

        let aabb = if surfaces.len() == 0 {
            MeshAABB {
                min: [0f32; 3],
                max: [0f32; 3],
            }
        } else {
            let mut aabb = MeshAABB {
                min: [f32::MAX; 3],
                max: [f32::MIN; 3],
            };

            for surface in surfaces {
                let vertex = &pmx.vertices[surface.vertex_indices[0].get() as usize];
                aabb.min[0] = aabb.min[0].min(vertex.position.x);
                aabb.min[1] = aabb.min[1].min(vertex.position.y);
                aabb.min[2] = aabb.min[2].min(vertex.position.z);

                aabb.max[0] = aabb.max[0].max(vertex.position.x);
                aabb.max[1] = aabb.max[1].max(vertex.position.y);
                aabb.max[2] = aabb.max[2].max(vertex.position.z);

                let vertex = &pmx.vertices[surface.vertex_indices[1].get() as usize];
                aabb.min[0] = aabb.min[0].min(vertex.position.x);
                aabb.min[1] = aabb.min[1].min(vertex.position.y);
                aabb.min[2] = aabb.min[2].min(vertex.position.z);

                aabb.max[0] = aabb.max[0].max(vertex.position.x);
                aabb.max[1] = aabb.max[1].max(vertex.position.y);
                aabb.max[2] = aabb.max[2].max(vertex.position.z);

                let vertex = &pmx.vertices[surface.vertex_indices[2].get() as usize];
                aabb.min[0] = aabb.min[0].min(vertex.position.x);
                aabb.min[1] = aabb.min[1].min(vertex.position.y);
                aabb.min[2] = aabb.min[2].min(vertex.position.z);

                aabb.max[0] = aabb.max[0].max(vertex.position.x);
                aabb.max[1] = aabb.max[1].max(vertex.position.y);
                aabb.max[2] = aabb.max[2].max(vertex.position.z);
            }

            aabb
        };

        let additional_vec4_count = pmx.header.config.additional_vec4_count;
        let mut vertex_attributes = Vec::with_capacity(3 + additional_vec4_count);
        vertex_attributes.push(VertexAttribute {
            offset: 0,
            kind: VertexAttributeKind::Position,
        });
        vertex_attributes.push(VertexAttribute {
            offset: size_of::<[f32; 3]>() as u32,
            kind: VertexAttributeKind::Normal,
        });
        vertex_attributes.push(VertexAttribute {
            offset: size_of::<[f32; 6]>() as u32,
            kind: VertexAttributeKind::TexCoord { index: 0 },
        });

        for index in 0..additional_vec4_count {
            vertex_attributes.push(VertexAttribute {
                offset: size_of::<[f32; 8]>() as u32 + (size_of::<[f32; 4]>() * index) as u32,
                kind: VertexAttributeKind::Extra {
                    index: index as u32,
                },
            });
        }

        let mut vertices = Vec::<u8>::with_capacity(
            surfaces.len() * 3 * (8 + additional_vec4_count * 4) * size_of::<f32>(),
        );
        let mut indices = Vec::with_capacity(surfaces.len() * 3);
        let mut index_map = HashMap::new();

        for (surface_index, surface) in surfaces.iter().enumerate() {
            for (surface_sub_index, vertex_index) in surface.vertex_indices.iter().enumerate() {
                let index = match index_map.entry(surface_index * 3 + surface_sub_index) {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        let vertex = &pmx.vertices[vertex_index.get() as usize];

                        vertices.extend_from_slice(
                            &[vertex.position.x, vertex.position.y, vertex.position.z].as_bytes(),
                        );
                        vertices.extend_from_slice(
                            &[vertex.normal.x, vertex.normal.y, vertex.normal.z].as_bytes(),
                        );
                        vertices.extend_from_slice(&[vertex.uv.x, vertex.uv.y].as_bytes());

                        for index in 0..additional_vec4_count {
                            let vec4 = &vertex.additional_vec4s[index];
                            vertices
                                .extend_from_slice(&[vec4.x, vec4.y, vec4.z, vec4.w].as_bytes());
                        }

                        let index = vertices.len() as u32;
                        entry.insert(index);
                        index
                    }
                };

                indices.push(index);
            }
        }

        // reduce vertex indices if possible
        let (index_type, indices) = if index_map.is_empty() {
            (VertexIndexType::U8, vec![])
        } else {
            let max = *index_map.values().max().unwrap();

            if max <= u8::MAX as u32 {
                let mut raw_indices = Vec::with_capacity(indices.len());

                for index in indices {
                    raw_indices.push(index as u8);
                }

                (VertexIndexType::U8, raw_indices)
            } else if max <= u16::MAX as u32 {
                let mut raw_indices = Vec::with_capacity(indices.len() * 2);

                for index in indices {
                    let index = index as u16;
                    raw_indices.extend_from_slice(&index.to_le_bytes());
                }

                (VertexIndexType::U16, raw_indices)
            } else {
                let mut raw_indices = Vec::with_capacity(indices.len() * 4);

                for index in indices {
                    raw_indices.extend_from_slice(&index.to_le_bytes());
                }

                (VertexIndexType::U32, raw_indices)
            }
        };

        meshes.push(MeshSource {
            index: material_index as u32,
            aabb,
            index_type,
            index_buffer: indices,
            vertex_attributes,
            vertex_buffer: vertices,
            vertex_count: surfaces.len() as u32 * 3,
            material: None,
        });
    }

    // test: drop all bones and attach all meshes to root node
    let nodes = vec![NodeSource {
        index: 0,
        parent_index: None,
        children_indices: vec![],
        name: "root".to_owned(),
        transform: NodeTransform {
            matrix: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        },
        mesh_indices: (0..meshes.len() as u32).collect(),
    }];

    Ok(ModelSource {
        root_node_index: Some(0),
        nodes,
        meshes,
    })
}

fn process_assimp_model(content: &[u8]) -> anyhow::Result<ModelSource> {
    let scene = Scene::from_buffer(
        &content,
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

    Ok(ModelSource {
        root_node_index,
        nodes,
        meshes,
    })
}

#[derive(Default)]
struct SceneExtractor {
    pub nodes: Vec<NodeSource>,
    pub meshes: Vec<MeshSource>,
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
        self.nodes.push(NodeSource {
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

fn convert_mesh(index: u32, mesh: &russimp::mesh::Mesh) -> MeshSource {
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
            _ => unreachable!(),
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

    MeshSource {
        index,
        aabb,
        index_type,
        index_buffer: raw_index_buffer,
        vertex_attributes,
        vertex_buffer: raw_vertex_buffer,
        vertex_count: vertex_count as u32,
        material: None,
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
