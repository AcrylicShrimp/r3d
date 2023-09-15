use specs::{prelude::*, Component};

mod object_component;
mod object_handle;
mod object_hierarchy;
mod object_id;
mod object_id_allocator;
mod object_manager;
mod object_name_registry;

pub use object_component::*;
pub use object_handle::*;
pub use object_hierarchy::*;
pub use object_id::*;
pub use object_id_allocator::*;
pub use object_manager::*;
pub use object_name_registry::*;

#[derive(Debug, Clone, Copy, Component)]
#[storage(VecStorage)]
pub struct Object {
    entity: Entity,
    object_id: ObjectId,
}

impl Object {
    pub fn new(entity: Entity, object_id: ObjectId) -> Self {
        Self { entity, object_id }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn object_id(&self) -> ObjectId {
        self.object_id
    }
}
