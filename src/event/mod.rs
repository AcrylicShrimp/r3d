use std::any::Any;

mod event_bus;
mod event_dispatcher;
mod event_handler;
pub mod event_types;

pub use event_bus::*;
pub use event_dispatcher::*;
pub use event_handler::*;

pub struct EventManager {
    bus: EventBus,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            bus: EventBus::new(),
        }
    }

    pub fn add_handler<T: Any>(&self, handler: EventHandler<T>) {
        self.bus.add_handler(handler);
    }

    pub fn remove_handler<T: Any>(&self, handler_id: EventHandlerId) {
        self.bus.remove_handler::<T>(handler_id);
    }

    pub fn dispatch<T: Any>(&self, event: &T) {
        self.bus.dispatch::<T>(event);
    }
}
