use super::RawInput;

pub struct RawInputEventDispatcher {}

impl RawInputEventDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn dispatch(&mut self, raw_input: &RawInput) {}
}
