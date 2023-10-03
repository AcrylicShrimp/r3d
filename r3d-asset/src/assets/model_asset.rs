use crate::{
    Asset, AssetDepsProvider, AssetLoadError, AssetSource, GfxBridge, GfxBuffer, TypedAsset,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use wgpu::BufferUsages;

/// Index element type of a mesh.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexIndexType {
    U8,
    U16,
    U32,
}

/// Type of a vertex attribute.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttributeKind {
    Position,
    Normal,
    Color { index: u32 },
    TexCoord { index: u32 },
    Tangent,
    Bitangent,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    /// In bytes.
    pub offset: u32,
    pub kind: VertexAttributeKind,
}

/// Bounding box of a sub mesh. This is un-transformed; in other words, it is in the local space of each sub mesh.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshAABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Transform of a sub mesh represented as a 4x4 matrix.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeTransform {
    pub matrix: [f32; 16],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub index: u32,
    pub parent_index: Option<u32>,
    pub children_indices: Vec<u32>,
    pub name: String,
    pub transform: NodeTransform,
    pub mesh_indices: Vec<u32>,
}

#[derive(Debug)]
pub struct Mesh {
    pub index: u32,
    pub aabb: MeshAABB,
    pub index_type: VertexIndexType,
    pub index_buffer: GfxBuffer,
    pub vertex_attributes: Vec<VertexAttribute>,
    pub vertex_buffer: GfxBuffer,
    pub vertex_count: u32,
    // TODO: Should we add a (mesh) material here?
}

/// Represents a mesy asset.
pub trait ModelAsset: Asset {
    fn root_node_index(&self) -> Option<u32>;
    fn nodes(&self) -> &[Node];
    fn meshes(&self) -> &[Mesh];
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshSource {
    pub index: u32,
    pub aabb: MeshAABB,
    pub index_type: VertexIndexType,
    /// Little-endian.
    pub index_buffer: Vec<u8>,
    pub vertex_attributes: Vec<VertexAttribute>,
    /// Little-endian.
    pub vertex_buffer: Vec<u8>,
    pub vertex_count: u32,
    // TODO: Should we add a (mesh) material here?
}

pub type NodeSource = Node;

#[derive(Serialize, Deserialize)]
pub struct ModelSource {
    pub root_node_index: Option<u32>,
    pub nodes: Vec<NodeSource>,
    pub meshes: Vec<MeshSource>,
}

impl AssetSource for ModelSource {
    type Asset = dyn ModelAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }

    fn load(
        self,
        id: Uuid,
        _deps_provider: &dyn AssetDepsProvider,
        gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        Ok(Arc::new(Model {
            id,
            root_node_index: self.root_node_index,
            nodes: self.nodes,
            meshes: self
                .meshes
                .into_iter()
                .map(|mesh| Mesh {
                    index: mesh.index,
                    aabb: mesh.aabb,
                    index_type: mesh.index_type,
                    index_buffer: gfx_bridge
                        .upload_vertex_buffer(BufferUsages::INDEX, &mesh.index_buffer),
                    vertex_attributes: mesh.vertex_attributes,
                    vertex_buffer: gfx_bridge
                        .upload_vertex_buffer(BufferUsages::VERTEX, &mesh.vertex_buffer),
                    vertex_count: mesh.vertex_count,
                })
                .collect(),
        }))
    }
}

struct Model {
    id: Uuid,
    root_node_index: Option<u32>,
    nodes: Vec<Node>,
    meshes: Vec<Mesh>,
}

impl Asset for Model {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Model(self)
    }
}

impl ModelAsset for Model {
    fn root_node_index(&self) -> Option<u32> {
        self.root_node_index
    }

    fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }
}
