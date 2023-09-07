use super::{BindGroupLayoutCache, Color, ScreenManager};
use crate::math::Mat4;
use specs::{prelude::*, Component};
use std::{mem::size_of, sync::Arc};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, Buffer, BufferAddress, BufferBinding, BufferBindingType, BufferDescriptor,
    BufferSize, BufferUsages, Device, Queue, ShaderStages,
};
use zerocopy::AsBytes;

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

    pub fn perspective(
        fov: f32,
        aspect: CameraPerspectiveProjectionAspect,
        near: f32,
        far: f32,
    ) -> Self {
        Self::Perspective(CameraPerspectiveProjection {
            fov,
            aspect,
            near,
            far,
        })
    }

    pub fn as_matrix(&self, screen_mgr: &ScreenManager) -> Mat4 {
        match self {
            Self::Orthographic(projection) => projection.as_matrix(),
            Self::Perspective(projection) => projection.as_matrix(screen_mgr),
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
    pub aspect: CameraPerspectiveProjectionAspect,
    pub near: f32,
    pub far: f32,
}

impl CameraPerspectiveProjection {
    pub fn as_matrix(&self, screen_mgr: &ScreenManager) -> Mat4 {
        Mat4::perspective(
            self.fov,
            match self.aspect {
                CameraPerspectiveProjectionAspect::Screen => {
                    screen_mgr.width() as f32 / screen_mgr.height() as f32
                }
                CameraPerspectiveProjectionAspect::Fixed(aspect) => aspect,
            },
            self.near,
            self.far,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CameraPerspectiveProjectionAspect {
    Screen,
    Fixed(f32),
}

#[derive(Debug, Clone, Component)]
#[storage(HashMapStorage)]
pub struct Camera {
    pub mask: u32,
    pub clear_mode: CameraClearMode,
    pub projection: CameraProjection,
    pub buffer: Arc<Buffer>,
    pub bind_group: Arc<BindGroup>,
}

impl Camera {
    pub fn new(
        mask: u32,
        clear_mode: CameraClearMode,
        projection: CameraProjection,
        device: &Device,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
    ) -> Self {
        let buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some("camera transform buffer"),
            size: size_of::<[f32; 4 * 4]>() as BufferAddress,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }));
        let bind_group = Arc::new(
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("camera transform bind group"),
                layout: bind_group_layout_cache
                    .create_layout(vec![BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                BufferSize::new(size_of::<[f32; 4 * 4]>() as u64).unwrap(),
                            ),
                        },
                        count: None,
                    }])
                    .as_ref(),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            }),
        );

        Self {
            mask,
            clear_mode,
            projection,
            buffer,
            bind_group,
        }
    }

    pub fn update_buffer(
        &self,
        screen_mgr: &ScreenManager,
        queue: &Queue,
        transform_matrix: &Mat4,
    ) {
        queue.write_buffer(
            &self.buffer,
            0,
            (transform_matrix.inversed() * self.projection.as_matrix(screen_mgr)).as_bytes(),
        );
    }
}
