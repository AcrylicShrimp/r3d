use crate::object::ObjectId;
use std::any::Any;

mod object_event_bus;
mod object_event_dispatcher;
mod object_event_handler;

pub use object_event_bus::*;
pub use object_event_dispatcher::*;
pub use object_event_handler::*;

pub struct ObjectEventManager {
    bus: ObjectEventBus,
}

impl ObjectEventManager {
    pub fn new() -> Self {
        Self {
            bus: ObjectEventBus::new(),
        }
    }

    pub fn add_handler<T: Any>(&self, handler: ObjectEventHandler<T>) {
        self.bus.add_handler(handler);
    }

    pub fn remove_handler(&self, handler_id: ObjectEventHandlerId) {
        self.bus.remove_handler(handler_id);
    }

    pub fn dispatch<T: Any>(&self, object_id: ObjectId, event: &T) {
        self.bus.dispatch::<T>(object_id, event);
    }
}
