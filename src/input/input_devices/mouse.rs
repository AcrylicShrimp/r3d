use crate::input::{InputDevice, RawInput, RawInputEventDispatcher};
use std::collections::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
};

pub enum MouseWindowEvent {
    CursorMoved {
        position: PhysicalPosition<f64>,
    },
    MouseWheel {
        delta: MouseScrollDelta,
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
}

pub struct Mouse {
    inputs: Vec<RawInput>,
    input_names: HashMap<String, usize>,
    window_event_queue: Vec<MouseWindowEvent>,
}

impl Mouse {
    pub fn new() -> Self {
        let inputs = vec![
            RawInput::new("x"),
            RawInput::new("y"),
            RawInput::new("delta:x"),
            RawInput::new("delta:y"),
            RawInput::new("scroll:x"),
            RawInput::new("scroll:y"),
            RawInput::new("button:left"),
            RawInput::new("button:right"),
            RawInput::new("button:middle"),
        ];
        let input_names = inputs
            .iter()
            .enumerate()
            .map(|(i, input)| (input.name.clone(), i))
            .collect();

        Self {
            inputs,
            input_names,
            window_event_queue: Vec::new(),
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.window_event_queue.push(MouseWindowEvent::CursorMoved {
                    position: *position,
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.window_event_queue
                    .push(MouseWindowEvent::MouseWheel { delta: *delta });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.window_event_queue.push(MouseWindowEvent::MouseInput {
                    state: *state,
                    button: *button,
                });
            }
            _ => {}
        }
    }
}

impl InputDevice for Mouse {
    fn name(&self) -> &str {
        "mouse"
    }

    fn inputs(&self) -> &[RawInput] {
        &self.inputs
    }

    fn input(&self, name: &str) -> Option<&RawInput> {
        self.input_names.get(name).map(|&index| &self.inputs[index])
    }

    fn poll(&mut self, dispatcher: &mut RawInputEventDispatcher) {
        let mut is_delta_changed = false
            || self.inputs[self.input_names["delta:x"]].value != 0.0
            || self.inputs[self.input_names["delta:y"]].value != 0.0;
        let mut is_scroll_changed = false
            || self.inputs[self.input_names["scroll:x"]].value != 0.0
            || self.inputs[self.input_names["scroll:y"]].value != 0.0;

        // Reset delta values.
        self.inputs[self.input_names["delta:x"]].value = 0.0;
        self.inputs[self.input_names["delta:y"]].value = 0.0;

        self.inputs[self.input_names["scroll:x"]].value = 0.0;
        self.inputs[self.input_names["scroll:y"]].value = 0.0;

        for event in self.window_event_queue.drain(..) {
            match event {
                MouseWindowEvent::CursorMoved { position } => {
                    let x_index = self.input_names["x"];
                    let y_index = self.input_names["y"];

                    let x_delta = position.x as f32 - self.inputs[x_index].value;
                    let y_delta = position.y as f32 - self.inputs[y_index].value;

                    self.inputs[x_index].value = position.x as f32;
                    self.inputs[y_index].value = position.y as f32;

                    dispatcher.dispatch(&self.inputs[x_index]);
                    dispatcher.dispatch(&self.inputs[y_index]);

                    self.inputs[self.input_names["delta:x"]].value = x_delta;
                    self.inputs[self.input_names["delta:y"]].value = y_delta;

                    is_delta_changed = true;
                }
                MouseWindowEvent::MouseWheel { delta } => {
                    let scroll_x_index = self.input_names["scroll:x"];
                    let scroll_y_index = self.input_names["scroll:y"];

                    match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            self.inputs[scroll_x_index].value += x;
                            self.inputs[scroll_y_index].value += y;
                        }
                        MouseScrollDelta::PixelDelta(position) => {
                            self.inputs[scroll_x_index].value += position.x as f32;
                            self.inputs[scroll_y_index].value += position.y as f32;
                        }
                    }

                    is_scroll_changed = true;
                }
                MouseWindowEvent::MouseInput { state, button } => {
                    let button_index = match button {
                        MouseButton::Left => self.input_names["button:left"],
                        MouseButton::Right => self.input_names["button:right"],
                        MouseButton::Middle => self.input_names["button:middle"],
                        _ => return,
                    };

                    match state {
                        ElementState::Pressed => self.inputs[button_index].value = 1.0,
                        ElementState::Released => self.inputs[button_index].value = 0.0,
                    }

                    dispatcher.dispatch(&self.inputs[button_index]);
                }
            }
        }

        if is_delta_changed {
            dispatcher.dispatch(&self.inputs[self.input_names["delta:x"]]);
            dispatcher.dispatch(&self.inputs[self.input_names["delta:y"]]);
        }

        if is_scroll_changed {
            dispatcher.dispatch(&self.inputs[self.input_names["scroll:x"]]);
            dispatcher.dispatch(&self.inputs[self.input_names["scroll:y"]]);
        }
    }
}
