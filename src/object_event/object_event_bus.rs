use super::{
    ObjectEventDispatcher, ObjectEventHandler, ObjectEventHandlerId, UntypedObjectEventDispatcher,
};
use crate::object::ObjectId;
use parking_lot::Mutex;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

struct UntypedDispatcher {
    dispatcher: Arc<dyn UntypedObjectEventDispatcher>,
}

impl UntypedDispatcher {
    pub fn new<T: Any>() -> Self {
        Self {
            dispatcher: Arc::new(ObjectEventDispatcher::<T>::new()),
        }
    }

    pub fn dispatcher(&self) -> &dyn UntypedObjectEventDispatcher {
        &*self.dispatcher
    }

    pub fn as_typed<T: Any>(&self) -> Option<&ObjectEventDispatcher<T>> {
        self.dispatcher
            .as_any()
            .downcast_ref::<ObjectEventDispatcher<T>>()
    }
}

impl Clone for UntypedDispatcher {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

pub struct ObjectEventBus {
    dispatchers: Mutex<HashMap<TypeId, UntypedDispatcher>>,
}

impl ObjectEventBus {
    pub fn new() -> Self {
        Self {
            dispatchers: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_handler<T: Any>(&self, handler: ObjectEventHandler<T>) -> ObjectEventHandlerId {
        let id = handler.id();
        let dispatcher = self
            .dispatchers
            .lock()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| UntypedDispatcher::new::<T>())
            .clone();

        dispatcher.as_typed::<T>().unwrap().add_handler(handler);

        id
    }

    pub fn remove_handler(&self, handler_id: ObjectEventHandlerId) {
        let dispatcher =
            if let Some(dispatcher) = self.dispatchers.lock().get(&handler_id.type_id()) {
                dispatcher.clone()
            } else {
                return;
            };

        dispatcher.dispatcher().remove_untyped_handler(handler_id);
    }

    pub fn remove_handler_for(&self, object_id: ObjectId) {
        let dispatchers = self.dispatchers.lock();

        for dispatcher in dispatchers.values() {
            dispatcher
                .dispatcher()
                .remove_untyped_handler_for(object_id);
        }
    }

    pub fn dispatch<T: Any>(&self, object_id: ObjectId, event: &T) {
        let dispatcher = if let Some(dispatcher) = self.dispatchers.lock().get(&TypeId::of::<T>()) {
            dispatcher.clone()
        } else {
            return;
        };

        dispatcher
            .as_typed::<T>()
            .unwrap()
            .dispatch(object_id, event);
    }
}
