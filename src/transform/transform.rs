use crate::math::{Mat4, Quat, Vec3};
use specs::{prelude::*, Component};

#[derive(Debug, Clone, Component)]
#[storage(VecStorage)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_mat4(matrix: &Mat4) -> Self {
        let (position, rotation, scale) = matrix.split();
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::srt(self.position, self.rotation, self.scale)
    }

    pub fn inverse_matrix(&self) -> Mat4 {
        Mat4::trs(-self.position, -self.rotation, Vec3::recip(self.scale))
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
        }
    }
}
