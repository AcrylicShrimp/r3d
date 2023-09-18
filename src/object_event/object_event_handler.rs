use crate::object::{Object, ObjectId};
use std::any::{Any, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectEventHandlerId {
    type_id: TypeId,
    object_id: ObjectId,
    ptr: *const (),
}

impl ObjectEventHandlerId {
    pub fn type_id(self) -> TypeId {
        self.type_id
    }

    pub fn object_id(self) -> ObjectId {
        self.object_id
    }
}

pub struct ObjectEventHandler<T: Any> {
    object: Object,
    closure: Box<dyn FnMut(Object, &T)>,
}

impl<T: Any> ObjectEventHandler<T> {
    pub fn new(object: Object, closure: impl FnMut(Object, &T) + 'static) -> Self {
        Self {
            object,
            closure: Box::new(closure),
        }
    }

    pub fn id(&self) -> ObjectEventHandlerId {
        ObjectEventHandlerId {
            type_id: TypeId::of::<T>(),
            object_id: self.object.object_id(),
            ptr: self.closure.as_ref() as *const _ as *const (),
        }
    }

    pub fn object(&self) -> Object {
        self.object
    }

    pub fn call(&mut self, event: &T) {
        (self.closure)(self.object, event);
    }
}
