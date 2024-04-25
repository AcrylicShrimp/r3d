use super::{new::Component, ComponentId, ObjectId};
use std::marker::PhantomData;

pub struct ComponentHandle<T: Component> {
    object: ObjectId,
    component: ComponentId,
    _phantom: PhantomData<T>,
}

impl<T: Component> ComponentHandle<T> {
    pub fn new(object: ObjectId, component: ComponentId) -> Self {
        Self {
            object,
            component,
            _phantom: PhantomData,
        }
    }
}
