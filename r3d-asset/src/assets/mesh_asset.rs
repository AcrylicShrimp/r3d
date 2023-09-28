use crate::{Asset, AssetDepsProvider, AssetLoadError, AssetSource, TypedAsset};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Index element type of a mesh.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshVertexIndexType {
    U8,
    U16,
    U32,
}

/// Type of a vertex attribute.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttributeKind {
    Position,
    Normal,
    TexCoord,
    TexCoord2,
    TexCoord3,
    TexCoord4,
    Color,
    Tangent,
    Bitangent,
    BoneWeight,
    BoneIndex,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshVertexAttribute {
    pub offset: u32,
    pub kind: VertexAttributeKind,
}

/// Bounding box of a sub mesh. This is un-transformed; in other words, it is in the local space of each sub mesh.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubMeshAABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Transform of a sub mesh represented as a 4x4 matrix.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubMeshTransform {
    pub matrix: [f32; 16],
}

/// Parts of a mesh. A mesh can be split into multiple sub meshes.
/// This sub mesh is the smallest unit of a mesh that can be rendered.
/// Sub meshes have hierachical relationships. For example, a sub mesh can be a child of another sub mesh.
/// In such case, parent's transform should be applied to the child's transform.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubMesh {
    pub index: u32,
    pub parent_index: Option<u32>,
    pub name: Option<String>,
    pub aabb: SubMeshAABB,
    pub transform: SubMeshTransform,
    pub index_buffer: Vec<u8>,
    pub vertex_count: u32,
    // TODO: Should we add a (mesh) material here?
}

/// Represents a mesy asset.
pub trait MeshAsset: Asset {
    fn index_type(&self) -> MeshVertexIndexType;
    fn vertex_attributes(&self) -> &[MeshVertexAttribute];
    fn vertex_buffer(&self) -> &[u8];
    fn sub_meshs(&self) -> &[SubMesh];
}

#[derive(Serialize, Deserialize)]
pub struct MeshSource {
    index_type: MeshVertexIndexType,
    vertex_attributes: Vec<MeshVertexAttribute>,
    vertex_buffer: Vec<u8>,
    sub_meshs: Vec<SubMesh>,
}

impl AssetSource for MeshSource {
    type Asset = dyn MeshAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }

    fn load(
        self,
        id: Uuid,
        _deps_provider: &dyn AssetDepsProvider,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        Ok(Arc::new(Mesh {
            id,
            index_type: self.index_type,
            vertex_attributes: self.vertex_attributes,
            vertex_buffer: self.vertex_buffer,
            sub_meshs: self.sub_meshs,
        }))
    }
}

struct Mesh {
    id: Uuid,
    index_type: MeshVertexIndexType,
    vertex_attributes: Vec<MeshVertexAttribute>,
    vertex_buffer: Vec<u8>,
    sub_meshs: Vec<SubMesh>,
}

impl Asset for Mesh {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Mesh(self)
    }
}

impl MeshAsset for Mesh {
    fn index_type(&self) -> MeshVertexIndexType {
        self.index_type
    }

    fn vertex_attributes(&self) -> &[MeshVertexAttribute] {
        &self.vertex_attributes
    }

    fn vertex_buffer(&self) -> &[u8] {
        &self.vertex_buffer
    }

    fn sub_meshs(&self) -> &[SubMesh] {
        &self.sub_meshs
    }
}
