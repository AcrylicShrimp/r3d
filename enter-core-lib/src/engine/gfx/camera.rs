use super::Color;
use crate::engine::math::Mat4;
use specs::{prelude::*, Component};

#[derive(Debug, Clone)]
pub enum CameraClearMode {
    Keep,
    All {
        color: Color,
        depth: f32,
        stencil: u32,
    },
    DepthOnly {
        depth: f32,
        stencil: u32,
    },
}

impl CameraClearMode {
    pub fn keep() -> Self {
        Self::Keep
    }

    pub fn all(color: Color, depth: f32, stencil: u32) -> Self {
        Self::All {
            color,
            depth,
            stencil,
        }
    }

    pub fn depth_only(depth: f32, stencil: u32) -> Self {
        Self::DepthOnly { depth, stencil }
    }
}

#[derive(Debug, Clone)]
pub enum CameraProjection {
    Orthographic(CamereOrthographicProjection),
    Perspective(CameraPerspectiveProjection),
}

impl CameraProjection {
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::Orthographic(CamereOrthographicProjection {
            left,
            right,
            bottom,
            top,
            near,
            far,
        })
    }

    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::Perspective(CameraPerspectiveProjection {
            fov,
            aspect,
            near,
            far,
        })
    }

    pub fn as_matrix(&self) -> Mat4 {
        match self {
            Self::Orthographic(projection) => projection.as_matrix(),
            Self::Perspective(projection) => projection.as_matrix(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CamereOrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl CamereOrthographicProjection {
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::orthographic(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CameraPerspectiveProjection {
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraPerspectiveProjection {
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov, self.aspect, self.near, self.far)
    }
}

#[derive(Debug, Clone, Component)]
#[storage(HashMapStorage)]
pub struct Camera {
    pub clear_mode: CameraClearMode,
    pub projection: CameraProjection,
}
