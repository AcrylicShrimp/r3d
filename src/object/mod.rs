use specs::{prelude::*, Component};

mod component_storage;
mod handle;
mod object_component;
mod object_handle;
mod object_hierarchy;
mod object_id;
mod object_id_allocator;
mod object_manager;
mod object_name_registry;
mod object_storage;

pub use component_storage::*;
pub use handle::*;
pub use object_component::*;
pub use object_handle::*;
pub use object_hierarchy::*;
pub use object_id::*;
pub use object_id_allocator::*;
pub use object_manager::*;
pub use object_name_registry::*;
pub use object_storage::*;

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

pub mod new {
    use super::{ComponentId, ComponentStorage, ObjectId};
    use std::cell::{Ref, RefMut};

    pub struct Object {
        component_ids: Vec<ComponentId>,
    }

    impl Object {
        pub fn new() -> Self {
            Self {
                component_ids: Vec::new(),
            }
        }

        pub fn component_ids(&self) -> &[ComponentId] {
            &self.component_ids
        }

        pub fn first_component_of_type<'a, T: Component>(
            &'a self,
            storage: &'a ComponentStorage,
        ) -> Option<Ref<'a, T>> {
            self.component_ids
                .iter()
                .find_map(|id| storage.get_component(*id))
        }

        pub fn first_component_of_type_mut<'a, T: Component>(
            &'a self,
            storage: &'a ComponentStorage,
        ) -> Option<RefMut<'a, T>> {
            self.component_ids
                .iter()
                .find_map(|id| storage.get_component_mut(*id))
        }

        pub fn components_of_type<'a, T: Component>(
            &'a self,
            storage: &'a ComponentStorage,
        ) -> impl Iterator<Item = Ref<'a, T>> + 'a {
            self.component_ids
                .iter()
                .filter_map(|id| storage.get_component(*id))
        }

        pub fn components_of_type_mut<'a, T: Component>(
            &'a self,
            storage: &'a ComponentStorage,
        ) -> impl Iterator<Item = RefMut<'a, T>> + 'a {
            self.component_ids
                .iter()
                .filter_map(|id| storage.get_component_mut(*id))
        }

        pub fn component_at<'a, T: Component>(
            &'a self,
            index: usize,
            storage: &'a ComponentStorage,
        ) -> Option<Ref<'a, T>> {
            self.component_ids
                .get(index)
                .and_then(|id| storage.get_component(*id))
        }

        pub fn component_at_mut<'a, T: Component>(
            &'a self,
            index: usize,
            storage: &'a ComponentStorage,
        ) -> Option<RefMut<'a, T>> {
            self.component_ids
                .get(index)
                .and_then(|id| storage.get_component_mut(*id))
        }

        pub fn add_component(
            &mut self,
            storage: &mut ComponentStorage,
            component: impl Component,
        ) -> ComponentId {
            let id = storage.add_component(component);
            self.component_ids.push(id);
            id
        }

        pub fn add_component_at(
            &mut self,
            storage: &mut ComponentStorage,
            index: usize,
            component: impl Component,
        ) -> ComponentId {
            let id = storage.add_component(component);
            self.component_ids.insert(index, id);
            id
        }

        pub fn remove_component<T: Component>(
            &mut self,
            storage: &mut ComponentStorage,
            id: ComponentId,
        ) {
            let index = if let Some(index) = self
                .component_ids
                .iter()
                .position(|component| *component == id)
            {
                index
            } else {
                return;
            };

            self.remove_component_at::<T>(storage, index);
        }

        pub fn remove_component_at<T: Component>(
            &mut self,
            storage: &mut ComponentStorage,
            index: usize,
        ) {
            let id = if let Some(id) = self.component_ids.get(index) {
                *id
            } else {
                return;
            };

            storage.remove_component::<T>(id);
            self.component_ids.swap_remove(index);
        }

        pub fn remove_components_of_type<T: Component>(&mut self, storage: &mut ComponentStorage) {
            let type_id = if let Some(type_id) = storage.get_type_id::<T>() {
                type_id
            } else {
                return;
            };

            let ids = self
                .component_ids
                .iter()
                .enumerate()
                .filter_map(|(index, id)| {
                    if (*id).type_id() == type_id {
                        Some((index, *id))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            for (index, id) in ids.into_iter().rev() {
                storage.remove_component::<T>(id);
                self.component_ids.swap_remove(index);
            }
        }
    }

    pub trait Component: 'static {
        fn new(id: ObjectId) -> Self
        where
            Self: Sized;

        /// Called when the component is added to an object.
        fn init(&mut self) {}
        /// Called when the component is removed from an object.
        fn fin(&mut self) {}
        /// Called when the object is enabled.
        fn enable(&mut self) {}
        /// Called when the object is disabled.
        fn disable(&mut self) {}
    }

    #[cfg(test)]
    mod tests {
        use component_storage::ComponentStorage;

        use super::{Component, Object};
        use crate::object::{component_storage, ObjectId, ObjectIdAllocator};

        struct TestComponent {
            value: i32,
        }

        impl Component for TestComponent {
            fn new(id: ObjectId) -> Self
            where
                Self: Sized,
            {
                todo!()
            }
        }

        impl TestComponent {
            pub fn new(value: i32) -> Self {
                Self { value }
            }

            pub fn value(&self) -> i32 {
                self.value
            }
        }

        // #[test]
        // fn test() {
        //     let mut component_storage = ComponentStorage::new();
        //     let mut object = Object::new();

        //     object.add_component(&mut component_storage, TestComponent::new(1));
        //     assert_eq!(object.component_ids().len(), 1);

        //     let component = object
        //         .first_component_of_type::<TestComponent>(&mut component_storage)
        //         .unwrap();
        //     assert_eq!(component.value(), 1);

        //     object.add_component(&mut component_storage, TestComponent::new(2));
        //     assert_eq!(object.component_ids().len(), 2);

        //     let component = object
        //         .first_component_of_type::<TestComponent>(&mut component_storage)
        //         .unwrap();
        //     assert_eq!(component.value(), 1);

        //     let component = object
        //         .first_component_of_type_mut::<TestComponent>(&mut component_storage)
        //         .unwrap();
        //     let pointer = component.pointer();

        //     object.remove_component(&mut component_storage, pointer);
        //     assert_eq!(object.component_ids().len(), 1);

        //     let component = object
        //         .first_component_of_type::<TestComponent>(&mut component_storage)
        //         .unwrap();
        //     assert_eq!(component.value(), 2);
        // }
    }
}
