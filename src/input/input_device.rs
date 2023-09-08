use super::{RawInput, RawInputEventDispatcher};

pub trait InputDevice {
    fn name(&self) -> &str;
    fn inputs(&self) -> &[RawInput];
    fn input(&self, name: &str) -> Option<&RawInput>;
    fn poll(&mut self, dispatcher: &mut RawInputEventDispatcher);
}
