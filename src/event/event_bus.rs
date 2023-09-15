use super::{EventDispatcher, EventHandler, EventHandlerId, UntypedEventDispatcher};
use parking_lot::Mutex;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

struct UntypedDispatcher {
    dispatcher: Arc<dyn UntypedEventDispatcher>,
}

impl UntypedDispatcher {
    pub fn new<T: Any>() -> Self {
        Self {
            dispatcher: Arc::new(EventDispatcher::<T>::new()),
        }
    }

    pub fn dispatcher(&self) -> &dyn UntypedEventDispatcher {
        &*self.dispatcher
    }

    pub fn as_typed<T: Any>(&self) -> Option<&EventDispatcher<T>> {
        self.dispatcher
            .as_any()
            .downcast_ref::<EventDispatcher<T>>()
    }
}

impl Clone for UntypedDispatcher {
    fn clone(&self) -> Self {
        Self {
            dispatcher: self.dispatcher.clone(),
        }
    }
}

pub struct EventBus {
    dispatchers: Mutex<HashMap<TypeId, UntypedDispatcher>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            dispatchers: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_handler<T: Any>(&self, handler: EventHandler<T>) -> EventHandlerId {
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

    pub fn remove_handler(&self, handler_id: EventHandlerId) {
        let dispatcher =
            if let Some(dispatcher) = self.dispatchers.lock().get(&handler_id.type_id()) {
                dispatcher.clone()
            } else {
                return;
            };

        dispatcher.dispatcher().remove_untyped_handler(handler_id);
    }

    pub fn dispatch<T: Any>(&self, event: &T) {
        let dispatcher = if let Some(dispatcher) = self.dispatchers.lock().get(&TypeId::of::<T>()) {
            dispatcher.clone()
        } else {
            return;
        };

        dispatcher.as_typed::<T>().unwrap().dispatch(event);
    }
}
