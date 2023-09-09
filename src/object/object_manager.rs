use super::{
    Object, ObjectHandle, ObjectHierarchy, ObjectId, ObjectIdAllocator, ObjectNameRegistry,
};
use crate::{transform::Transform, use_context};
use specs::prelude::*;

pub struct ObjectManager {
    world: World,
    object_hierarchy: ObjectHierarchy,
    object_name_registry: ObjectNameRegistry,
    object_id_allocator: ObjectIdAllocator,
}

impl ObjectManager {
    pub fn new() -> Self {
        let mut world = World::new();

        world.register::<Object>();
        world.register::<Transform>();

        Self {
            world,
            object_hierarchy: ObjectHierarchy::new(),
            object_name_registry: ObjectNameRegistry::new(),
            object_id_allocator: ObjectIdAllocator::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
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

    pub fn create_object_builder(
        &mut self,
        name: Option<String>,
        transform: Option<Transform>,
    ) -> (ObjectHandle, EntityBuilder) {
        let object_id = self.object_id_allocator.alloc();
        let builder = self.world.create_entity();
        let entity = builder.entity;

        self.object_hierarchy.add(object_id, entity);
        self.object_name_registry.set_name(object_id, name);

        let object_handle = ObjectHandle::new(use_context().clone(), entity, object_id);

        (
            object_handle,
            builder
                .with(Object::new(entity, object_id))
                .with(transform.unwrap_or_default()),
        )
    }

    pub fn remove_object(&mut self, handle: &ObjectHandle) {
        self.world.delete_entity(handle.entity).unwrap();
        self.object_hierarchy.remove(handle.object_id);
        self.object_id_allocator.dealloc(handle.object_id);
        self.object_name_registry.set_name(handle.object_id, None);
    }

    pub fn split(&self) -> (&World, &ObjectHierarchy) {
        (&self.world, &self.object_hierarchy)
    }

    pub fn split_mut(&mut self) -> (&mut World, &mut ObjectHierarchy) {
        (&mut self.world, &mut self.object_hierarchy)
    }
}
