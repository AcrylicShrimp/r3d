use std::any::{Any, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventHandlerId {
    type_id: TypeId,
    ptr: *const (),
}

impl EventHandlerId {
    pub fn type_id(self) -> TypeId {
        self.type_id
    }
}

pub struct EventHandler<T: Any> {
    closure: Box<dyn FnMut(&T)>,
}

impl<T: Any> EventHandler<T> {
    pub fn new(closure: impl FnMut(&T) + 'static) -> Self {
        Self {
            closure: Box::new(closure),
        }
    }

    pub fn id(&self) -> EventHandlerId {
        EventHandlerId {
            type_id: TypeId::of::<T>(),
            ptr: self.closure.as_ref() as *const _ as *const (),
        }
    }

    pub fn call(&mut self, event: &T) {
        (self.closure)(event);
    }
}
