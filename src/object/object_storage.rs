use self::new::ObjectId;
use super::{
    new::{Component, Object},
    ComponentId, ComponentStorage,
};
use crate::util::SlotMap;

pub mod new {
    pub type ObjectId = usize;
}

#[derive(Default)]
pub struct ObjectStorage {
    objects: SlotMap<Object>,
    component_storage: ComponentStorage,
}

impl ObjectStorage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn component_storage(&self) -> &ComponentStorage {
        &self.component_storage
    }

    pub fn add_object(&mut self) -> ObjectId {
        self.objects.allocate(Object::new())
    }

    pub fn remove_object(&mut self, id: ObjectId) -> Option<()> {
        let object = self.objects.get_mut(id)?;

        for id in object.component_ids() {
            self.component_storage.remove_component_untyped(*id);
        }

        self.objects.deallocate(id);

        Some(())
    }

    pub fn add_component(
        &mut self,
        id: ObjectId,
        component: impl Component,
    ) -> Option<ComponentId> {
        let object = self.objects.get_mut(id)?;
        Some(object.add_component(&mut self.component_storage, component))
    }

    pub fn add_component_at(
        &mut self,
        id: ObjectId,
        index: usize,
        component: impl Component,
    ) -> Option<ComponentId> {
        let object = self.objects.get_mut(id)?;
        Some(object.add_component_at(&mut self.component_storage, index, component))
    }

    pub fn remove_component(&mut self, id: ObjectId, component_id: ComponentId) {
        if let Some(object) = self.objects.get_mut(id) {
            // TODO: we need a method that only removes the component from the object,
            // but not from the component storage
            object.remove_component(&mut self.component_storage, component_id);
        }
    }
}
