use codegen::Handle;
use russimp::mesh::Mesh as RussimpMesh;

#[derive(Handle)]
pub struct Mesh {
    pub data: RussimpMesh,
}
