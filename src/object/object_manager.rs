use super::{
    Object, ObjectHandle, ObjectHierarchy, ObjectId, ObjectIdAllocator, ObjectNameRegistry,
};
use crate::{transform::Transform, use_context};
use specs::prelude::*;

pub struct ObjectManager {
    object_hierarchy: ObjectHierarchy,
    object_name_registry: ObjectNameRegistry,
    object_id_allocator: ObjectIdAllocator,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            object_hierarchy: ObjectHierarchy::new(),
            object_name_registry: ObjectNameRegistry::new(),
            object_id_allocator: ObjectIdAllocator::new(),
        }
    }

    pub fn object_name_registry(&self) -> &ObjectNameRegistry {
        &self.object_name_registry
    }

    pub fn object_name_registry_mut(&mut self) -> &mut ObjectNameRegistry {
        &mut self.object_name_registry
    }

    pub fn object_hierarchy(&self) -> &ObjectHierarchy {
        &self.object_hierarchy
    }

    pub fn object_hierarchy_mut(&mut self) -> &mut ObjectHierarchy {
        &mut self.object_hierarchy
    }

    pub fn object_handle(&self, object_id: ObjectId) -> ObjectHandle {
        ObjectHandle::new(
            use_context().clone(),
            self.object_hierarchy.entity(object_id),
            object_id,
        )
    }

    pub fn find(&self, name: &str) -> Option<ObjectHandle> {
        self.object_name_registry
            .ids(name)
            .and_then(|mut ids| ids.next())
            .map(|id| self.object_handle(id))
    }

    pub fn find_all(&self, name: &str) -> Vec<ObjectHandle> {
        self.object_name_registry
            .ids(name)
            .map(|ids| ids.map(|id| self.object_handle(id)).collect())
            .unwrap_or_default()
    }

    pub fn create_object_builder<'w>(
        &mut self,
        world: &'w mut World,
        name: impl Into<Option<String>>,
        transform: Option<Transform>,
    ) -> (ObjectHandle, EntityBuilder<'w>) {
        let object_id = self.object_id_allocator.alloc();
        let builder = world.create_entity();
        let entity = builder.entity;

        self.object_hierarchy.add(object_id, entity);
        self.object_name_registry.set_name(object_id, name.into());

        let object_handle = ObjectHandle::new(use_context().clone(), entity, object_id);

        (
            object_handle,
            builder
                .with(Object::new(entity, object_id))
                .with(transform.unwrap_or_default()),
        )
    }

    pub fn remove_object(&mut self, handle: &ObjectHandle) {
        use_context()
            .world_mut()
            .delete_entity(handle.entity)
            .unwrap();
        self.object_hierarchy.remove(handle.object_id);
        self.object_id_allocator.dealloc(handle.object_id);
        self.object_name_registry.set_name(handle.object_id, None);
    }
}
