use crate::math::Vec2;
use specs::{prelude::*, Component};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UIScaleMode {
    Constant,
    Stretch,
    Fit,
    Fill,
    MatchWidth,
    MatchHeight,
}

#[derive(Component, Debug, Clone)]
#[storage(HashMapStorage)]
pub struct UIScaler {
    pub mode: UIScaleMode,
    pub reference_size: Vec2,
}
