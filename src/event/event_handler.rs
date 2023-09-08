use crate::ContextHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventHandlerId(usize);

pub struct EventHandler<T> {
    closure: Box<dyn FnMut(&ContextHandle, &T)>,
}

impl<T> EventHandler<T> {
    pub fn new(closure: impl FnMut(&ContextHandle, &T) + 'static) -> Self {
        Self {
            closure: Box::new(closure),
        }
    }

    pub fn id(&self) -> EventHandlerId {
        EventHandlerId(self.closure.as_ref() as *const _ as *const () as usize)
    }

    pub fn call(&mut self, ctx: &ContextHandle, event: &T) {
        (self.closure)(ctx, event);
    }
}
