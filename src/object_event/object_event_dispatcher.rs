use super::{ObjectEventHandler, ObjectEventHandlerId};
use crate::object::ObjectId;
use parking_lot::Mutex;
use std::{any::Any, collections::HashMap};

pub trait UntypedObjectEventDispatcher: Any {
    fn as_any(&self) -> &dyn Any;

    fn remove_untyped_handler(&self, handler_id: ObjectEventHandlerId);

    fn remove_untyped_handler_for(&self, object_id: ObjectId);
}

pub struct ObjectEventDispatcher<T: Any> {
    handlers: Mutex<HashMap<ObjectId, Vec<ObjectEventHandler<T>>>>,
    added_queue: Mutex<Vec<ObjectEventHandler<T>>>,
    removed_queue: Mutex<Vec<ObjectEventHandlerId>>,
    removed_queue_for: Mutex<Vec<ObjectId>>,
}

impl<T: Any> ObjectEventDispatcher<T> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new().into(),
            added_queue: Vec::new().into(),
            removed_queue: Vec::new().into(),
            removed_queue_for: Vec::new().into(),
        }
    }

    pub fn add_handler(&self, handler: ObjectEventHandler<T>) {
        match self.handlers.try_lock() {
            Some(mut handlers) => {
                handlers
                    .entry(handler.object().object_id())
                    .or_default()
                    .push(handler);
            }
            None => {
                self.added_queue.lock().push(handler);
            }
        }
    }

    pub fn remove_handler(&self, handler_id: ObjectEventHandlerId) {
        match self.handlers.try_lock() {
            Some(mut handlers) => {
                let handlers = if let Some(handlers) = handlers.get_mut(&handler_id.object_id()) {
                    handlers
                } else {
                    return;
                };
                if let Some(index) = handlers
                    .iter()
                    .position(|handler| handler.id() == handler_id)
                {
                    handlers.swap_remove(index);
                }
            }
            None => {
                self.removed_queue.lock().push(handler_id);
            }
        }
    }

    pub fn remove_handler_for(&self, object_id: ObjectId) {
        match self.handlers.try_lock() {
            Some(mut handlers) => {
                handlers.remove(&object_id);
            }
            None => {
                self.removed_queue_for.lock().push(object_id);
            }
        }
    }

    pub fn dispatch(&self, object_id: ObjectId, event: &T) {
        let mut handlers = if let Some(handlers) = self.handlers.try_lock() {
            handlers
        } else {
            return;
        };

        {
            let handlers = if let Some(handlers) = handlers.get_mut(&object_id) {
                handlers
            } else {
                return;
            };

            for handler in handlers.iter_mut() {
                handler.call(event);
            }

            for removed in self.removed_queue.lock().drain(..) {
                if let Some(index) = handlers.iter().position(|handler| handler.id() == removed) {
                    handlers.swap_remove(index);
                }
            }

            handlers.extend(self.added_queue.lock().drain(..));
        }

        for removed in self.removed_queue_for.lock().drain(..) {
            handlers.remove(&removed);
        }
    }
}

impl<T: Any> UntypedObjectEventDispatcher for ObjectEventDispatcher<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn remove_untyped_handler(&self, handler_id: ObjectEventHandlerId) {
        self.remove_handler(handler_id);
    }

    fn remove_untyped_handler_for(&self, object_id: ObjectId) {
        self.remove_handler_for(object_id);
    }
}
