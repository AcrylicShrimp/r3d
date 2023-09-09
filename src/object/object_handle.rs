use super::{ObjectComponent, ObjectId};
use crate::ContextHandle;
use specs::Entity;

#[derive(Clone)]
pub struct ObjectHandle {
    pub ctx: ContextHandle,
    pub entity: Entity,
    pub object_id: ObjectId,
}

impl ObjectHandle {
    pub fn new(ctx: ContextHandle, entity: Entity, object_id: ObjectId) -> Self {
        Self {
            ctx,
            entity,
            object_id,
        }
    }

    pub fn component<T: ObjectComponent>(&self) -> T {
        T::new(self.clone())
    }

    pub fn is_active(&self) -> bool {
        self.ctx
            .object_mgr()
            .object_hierarchy()
            .is_active(self.object_id)
    }

    pub fn is_active_self(&self) -> bool {
        self.ctx
            .object_mgr()
            .object_hierarchy()
            .is_active_self(self.object_id)
    }

    pub fn name(&self) -> Option<String> {
        self.ctx
            .object_mgr()
            .object_name_registry()
            .name(self.object_id)
            .cloned()
    }

    pub fn parent(&self) -> Option<Self> {
        self.ctx
            .object_mgr()
            .object_hierarchy()
            .parent(self.object_id)
            .map(|parent_id| self.ctx.object_mgr().object_handle(parent_id))
    }

    pub fn parents(&self) -> Vec<Self> {
        self.ctx
            .object_mgr()
            .object_hierarchy()
            .parents(self.object_id)
            .iter()
            .map(|&parent_id| self.ctx.object_mgr().object_handle(parent_id))
            .collect()
    }

    pub fn children(&self) -> Vec<Self> {
        self.ctx
            .object_mgr()
            .object_hierarchy()
            .children(self.object_id)
            .iter()
            .map(|&child_id| self.ctx.object_mgr().object_handle(child_id))
            .collect()
    }

    pub fn direct_children(&self) -> Vec<Self> {
        match self
            .ctx
            .object_mgr()
            .object_hierarchy()
            .direct_children_iter(self.object_id)
        {
            Some(iter) => iter
                .map(|child_id| self.ctx.object_mgr().object_handle(child_id))
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn set_active(&self, active: bool) {
        self.ctx
            .object_mgr_mut()
            .object_hierarchy_mut()
            .set_active(self.object_id, active);
    }

    pub fn set_name(&self, name: impl Into<Option<String>>) {
        self.ctx
            .object_mgr_mut()
            .object_name_registry_mut()
            .set_name(self.object_id, name.into());
    }

    pub fn set_parent<'a>(&self, parent: impl Into<Option<&'a Self>>) {
        self.ctx
            .object_mgr_mut()
            .object_hierarchy_mut()
            .set_parent(self.object_id, parent.into().map(|parent| parent.object_id));
    }

    pub fn remove(&self) {
        self.ctx.object_mgr_mut().remove_object(self);
    }
}

impl PartialEq for ObjectHandle {
    fn eq(&self, other: &Self) -> bool {
        self.object_id == other.object_id
    }
}

impl Eq for ObjectHandle {}
