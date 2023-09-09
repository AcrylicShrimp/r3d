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
    mouse: Mouse,
    dispatcher: RawInputEventDispatcher,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            dispatcher: RawInputEventDispatcher::new(),
        }
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }

    pub fn mouse_mut(&mut self) -> &mut Mouse {
        &mut self.mouse
    }

    pub fn poll(&mut self) {
        self.keyboard.poll(&mut self.dispatcher);
        self.mouse.poll(&mut self.dispatcher);
    }
}
