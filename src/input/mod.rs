mod input_device;
mod input_devices;
mod raw_input;
mod raw_input_event;
mod raw_input_event_dispatcher;

pub use input_device::*;
pub use input_devices::*;
pub use raw_input::*;
pub use raw_input_event::*;
pub use raw_input_event_dispatcher::*;

pub struct InputManager {
    keyboard: Keyboard,
    dispatcher: RawInputEventDispatcher,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            dispatcher: RawInputEventDispatcher::new(),
        }
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    pub fn poll(&mut self) {
        self.keyboard.poll(&mut self.dispatcher);
    }
}
