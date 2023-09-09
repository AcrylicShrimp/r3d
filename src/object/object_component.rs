use super::ObjectHandle;
use specs::Component;

pub trait ObjectComponent {
    type Component: Component;

    fn new(object: ObjectHandle) -> Self;

    fn object(&self) -> &ObjectHandle;
}
