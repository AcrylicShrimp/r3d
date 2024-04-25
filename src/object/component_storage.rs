use super::new::Component;
use crate::util::SlotMap;
use downcast_rs::{impl_downcast, Downcast};
use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::{hash_map::Entry, HashMap},
    num::NonZeroU32,
};

pub type ComponentTypeId = NonZeroU32;
pub type Storage<T> = SlotMap<RefCell<T>>;

trait ComponentSubStorage: Downcast {
    fn remove_component_untyped(&mut self, id: usize);
}

impl_downcast!(ComponentSubStorage);

impl<T: Component> ComponentSubStorage for SlotMap<RefCell<T>> {
    fn remove_component_untyped(&mut self, id: usize) {
        self.deallocate(id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId {
    type_id: ComponentTypeId,
    component_id: u32,
}

impl ComponentId {
    pub fn type_id(self) -> ComponentTypeId {
        self.type_id
    }

    pub fn component_id(self) -> u32 {
        self.component_id
    }
}

#[derive(Default)]
pub struct ComponentStorage {
    type_id_type_map: HashMap<ComponentTypeId, TypeId>,
    type_type_id_map: HashMap<TypeId, ComponentTypeId>,
    storages: HashMap<ComponentTypeId, Box<dyn ComponentSubStorage>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_type_id<T: Component>(&self) -> Option<ComponentTypeId> {
        self.type_type_id_map.get(&TypeId::of::<T>()).copied()
    }

    pub fn get_component<T: Component>(&self, id: ComponentId) -> Option<Ref<T>> {
        let storage = self.storages.get(&id.type_id)?;
        let storage = storage.downcast_ref::<Storage<T>>()?;
        storage
            .get(id.component_id as usize)
            .map(|cell| cell.borrow())
    }

    pub fn get_component_mut<T: Component>(&self, id: ComponentId) -> Option<RefMut<T>> {
        let storage = self.storages.get(&id.type_id)?;
        let storage = storage.downcast_ref::<Storage<T>>()?;
        storage
            .get(id.component_id as usize)
            .map(|cell| cell.borrow_mut())
    }

    pub fn add_component<T: Component>(&mut self, component: T) -> ComponentId {
        let type_id = match self.type_type_id_map.entry(TypeId::of::<T>()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let type_id = unsafe {
                    ComponentTypeId::new_unchecked(self.type_id_type_map.len() as u32 + 1)
                };
                entry.insert(type_id);
                self.type_id_type_map.insert(type_id, TypeId::of::<T>());
                type_id
            }
        };

        let storage = self
            .storages
            .entry(type_id)
            .or_insert_with(|| Box::new(Storage::<T>::new()));
        let component_id = storage
            .downcast_mut::<Storage<T>>()
            .unwrap()
            .allocate(component.into()) as u32;

        ComponentId {
            type_id,
            component_id,
        }
    }

    pub fn remove_component<T: Component>(&mut self, id: ComponentId) {
        let storage = if let Some(stoage) = self.storages.get_mut(&id.type_id) {
            stoage
        } else {
            return;
        };

        storage
            .downcast_mut::<Storage<T>>()
            .unwrap()
            .deallocate(id.component_id as usize);
    }

    pub fn remove_component_untyped(&mut self, id: ComponentId) {
        let storage = if let Some(stoage) = self.storages.get_mut(&id.type_id) {
            stoage
        } else {
            return;
        };

        storage.remove_component_untyped(id.component_id as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentStorage;
    use crate::object::{new::Component, ObjectId};

    struct TestComponentA {
        value: &'static str,
    }

    impl Component for TestComponentA {
        fn new(_id: ObjectId) -> Self
        where
            Self: Sized,
        {
            Self { value: "" }
        }
    }

    struct TestComponentB {
        value: &'static str,
    }

    impl Component for TestComponentB {
        fn new(_id: ObjectId) -> Self
        where
            Self: Sized,
        {
            Self { value: "" }
        }
    }

    #[test]
    fn test_component_storage() {
        let mut storage = ComponentStorage::new();

        let foo = storage.add_component(TestComponentA { value: "foo" });
        let bar = storage.add_component(TestComponentA { value: "bar" });
        let baz = storage.add_component(TestComponentA { value: "baz" });

        assert_eq!(
            storage.get_component::<TestComponentA>(foo).unwrap().value,
            "foo"
        );
        assert_eq!(
            storage.get_component::<TestComponentA>(bar).unwrap().value,
            "bar"
        );
        assert_eq!(
            storage.get_component::<TestComponentA>(baz).unwrap().value,
            "baz"
        );

        storage.remove_component::<TestComponentA>(bar);

        assert_eq!(
            storage.get_component::<TestComponentA>(foo).unwrap().value,
            "foo"
        );
        assert_eq!(
            storage.get_component::<TestComponentA>(baz).unwrap().value,
            "baz"
        );

        storage.remove_component::<TestComponentA>(foo);

        assert_eq!(
            storage.get_component::<TestComponentA>(baz).unwrap().value,
            "baz"
        );

        let qux = storage.add_component(TestComponentB { value: "qux" });
        let quux = storage.add_component(TestComponentB { value: "quux" });

        assert_eq!(
            storage.get_component::<TestComponentA>(baz).unwrap().value,
            "baz"
        );
        assert!(storage.get_component::<TestComponentB>(baz).is_none());

        assert!(storage.get_component::<TestComponentA>(qux).is_none());
        assert_eq!(
            storage.get_component::<TestComponentB>(qux).unwrap().value,
            "qux"
        );
        assert!(storage.get_component::<TestComponentA>(quux).is_none());
        assert_eq!(
            storage.get_component::<TestComponentB>(quux).unwrap().value,
            "quux"
        );

        storage.remove_component::<TestComponentB>(qux);

        assert_eq!(
            storage.get_component::<TestComponentA>(baz).unwrap().value,
            "baz"
        );
        assert!(storage.get_component::<TestComponentB>(baz).is_none());

        assert!(storage.get_component::<TestComponentA>(quux).is_none());
        assert_eq!(
            storage.get_component::<TestComponentB>(quux).unwrap().value,
            "quux"
        );

        storage.remove_component_untyped(quux);

        assert_eq!(
            storage.get_component::<TestComponentA>(baz).unwrap().value,
            "baz"
        );
        assert!(storage.get_component::<TestComponentB>(baz).is_none());

        assert!(storage.get_component::<TestComponentA>(quux).is_none());
        assert!(storage.get_component::<TestComponentB>(quux).is_none());

        storage.remove_component_untyped(baz);

        assert!(storage.get_component::<TestComponentA>(baz).is_none());
        assert!(storage.get_component::<TestComponentB>(baz).is_none());

        assert!(storage.get_component::<TestComponentA>(quux).is_none());
        assert!(storage.get_component::<TestComponentB>(quux).is_none());
    }
}
