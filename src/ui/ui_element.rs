use crate::math::Vec2;
use specs::{prelude::*, Component};

#[derive(Debug, Clone, PartialEq)]
pub struct UIAnchor {
    pub min: Vec2,
    pub max: Vec2,
}

impl UIAnchor {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn full() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UIMargin {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl UIMargin {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn from_size(pivot: Vec2, position: Vec2, size: Vec2) -> Self {
        let pivot_x = pivot.x * size.x;
        let pivot_y = pivot.y * size.y;

        Self {
            left: position.x - pivot_x,
            right: -(position.x - pivot_x + size.x),
            top: -(position.y - pivot_y + size.y),
            bottom: position.y - pivot_y,
        }
    }

    pub fn zero() -> Self {
        Self {
            left: 0f32,
            right: 0f32,
            top: 0f32,
            bottom: 0f32,
        }
    }
}

#[derive(Debug, Clone, Component)]
#[storage(HashMapStorage)]
pub struct UIElement {
    pub anchor: UIAnchor,
    pub margin: UIMargin,
    pub is_interactable: bool,
}

impl UIElement {
    pub fn new(anchor: UIAnchor, margin: UIMargin, is_interactable: bool) -> Self {
        Self {
            anchor,
            margin,
            is_interactable,
        }
    }
}

impl Default for UIElement {
    fn default() -> Self {
        Self {
            anchor: UIAnchor::full(),
            margin: UIMargin::zero(),
            is_interactable: false,
        }
    }
}
