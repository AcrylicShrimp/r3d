#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventHandlerId(usize);

pub struct EventHandler<T> {
    closure: Box<dyn FnMut(&T)>,
}

impl<T> EventHandler<T> {
    pub fn new(closure: impl FnMut(&T) + 'static) -> Self {
        Self {
            closure: Box::new(closure),
        }
    }

    pub fn id(&self) -> EventHandlerId {
        EventHandlerId(self.closure.as_ref() as *const _ as *const () as usize)
    }

    pub fn call(&mut self, event: &T) {
        (self.closure)(event);
    }
}
