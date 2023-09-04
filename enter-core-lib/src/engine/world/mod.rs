use super::{
    object::{Object, ObjectHierarchy, ObjectId, ObjectIdAllocator},
    transform::Transform,
};
use specs::prelude::*;

pub struct WorldManager {
    world: World,
    object_hierarchy: ObjectHierarchy,
    object_id_allocator: ObjectIdAllocator,
}

impl WorldManager {
    pub fn new() -> Self {
        let mut world = World::new();

        world.register::<Object>();
        world.register::<Transform>();

        Self {
            world,
            object_hierarchy: ObjectHierarchy::new(),
            object_id_allocator: ObjectIdAllocator::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn object_hierarchy(&self) -> &ObjectHierarchy {
        &self.object_hierarchy
    }

    pub fn object_hierarchy_mut(&mut self) -> &mut ObjectHierarchy {
        &mut self.object_hierarchy
    }

    pub fn create_object_builder(
        &mut self,
        transform: Option<Transform>,
    ) -> (ObjectId, EntityBuilder) {
        let object_id = self.object_id_allocator.alloc();
        let builder = self.world.create_entity();
        let entity = builder.entity;

        self.object_hierarchy.add(object_id, entity);

        (
            object_id,
            builder
                .with(Object::new(entity, object_id))
                .with(transform.unwrap_or_default()),
        )
    }

    pub fn split(&self) -> (&World, &ObjectHierarchy) {
        (&self.world, &self.object_hierarchy)
    }

    pub fn split_mut(&mut self) -> (&mut World, &mut ObjectHierarchy) {
        (&mut self.world, &mut self.object_hierarchy)
    }
}
