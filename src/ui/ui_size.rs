use specs::{prelude::*, Component};

use crate::math::Vec2;

#[derive(Debug, Clone, Component)]
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
