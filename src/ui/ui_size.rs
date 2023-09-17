use crate::{
    math::Vec2,
    object::{ObjectComponent, ObjectHandle},
};
use specs::{prelude::*, Component};

#[derive(Debug, Clone, Copy, Component)]
#[storage(HashMapStorage)]
pub struct UISize {
    pub width: f32,
    pub height: f32,
}

impl UISize {
    pub fn new() -> Self {
        Self {
            width: 0f32,
            height: 0f32,
        }
    }

    pub fn from_vec2(vec: Vec2) -> Self {
        Self {
            width: vec.x,
            height: vec.y,
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

pub struct UISizeComponent {
    object: ObjectHandle,
}

impl ObjectComponent for UISizeComponent {
    type Component = UISize;

    fn new(object: ObjectHandle) -> Self {
        Self { object }
    }

    fn object(&self) -> &ObjectHandle {
        &self.object
    }
}

impl UISizeComponent {
    pub fn size(&self) -> Vec2 {
        let world = self.object.ctx.world();
        let ui_sizes = world.read_storage::<UISize>();
        ui_sizes.get(self.object.entity).unwrap().to_vec2()
    }

    pub fn set_size(&self, width: f32, height: f32) {
        let world = self.object.ctx.world();
        let mut ui_sizes = world.write_storage::<UISize>();
        let ui_size = ui_sizes.get_mut(self.object.entity).unwrap();
        ui_size.width = width;
        ui_size.height = height;
    }
}
