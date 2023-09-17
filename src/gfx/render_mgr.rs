use super::{
    build_rendering_command, BindGroupLayoutCache, CameraClearMode, DepthStencil, DepthStencilMode,
    FrameBufferAllocator, GenericBufferAllocation, GfxContextHandle, PipelineCache,
    PipelineLayoutCache, Renderer, RenderingCommand,
};
use crate::object::{ObjectHierarchy, ObjectId};
use std::mem::size_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferSize, BufferUsages, Color, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, SurfaceError, TextureView,
};
use winit::dpi::PhysicalSize;
use zerocopy::AsBytes;

pub struct RenderManager {
    gfx_ctx: GfxContextHandle,
    depth_stencil: DepthStencil,
    bind_group_layout_cache: BindGroupLayoutCache,
    pipeline_layout_cache: PipelineLayoutCache,
    pipeline_cache: PipelineCache,
    frame_buffer_allocator: FrameBufferAllocator,
    standard_ui_vertex_buffer: GenericBufferAllocation<Buffer>,
}

impl RenderManager {
    pub fn new(
        gfx_ctx: GfxContextHandle,
        size: PhysicalSize<u32>,
        depth_stencil_mode: DepthStencilMode,
    ) -> Self {
        let depth_stencil = DepthStencil::new(gfx_ctx.clone(), depth_stencil_mode, size).unwrap();
        let bind_group_layout_cache = BindGroupLayoutCache::new(gfx_ctx.clone());
        let pipeline_layout_cache = PipelineLayoutCache::new(gfx_ctx.clone());
        let pipeline_cache = PipelineCache::new(gfx_ctx.clone());
        let frame_buffer_allocator = FrameBufferAllocator::new(gfx_ctx.clone());

        // Since ui elements are always left-bottom based, positions must in range [0, 1].
        let standard_ui_vertices = vec![
            0.0f32, 0.0f32, 0.0f32, // bottom left position
            1.0f32, 0.0f32, 0.0f32, // bottom right position
            1.0f32, 1.0f32, 0.0f32, // top right position
            0.0f32, 0.0f32, 0.0f32, // bottom left position
            1.0f32, 1.0f32, 0.0f32, // top right position
            0.0f32, 1.0f32, 0.0f32, // top left position
        ];
        let standard_ui_vertex_buffer = GenericBufferAllocation::new(
            gfx_ctx.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: standard_ui_vertices.as_bytes(),
                usage: BufferUsages::VERTEX,
            }),
            0,
            BufferSize::new((size_of::<f32>() * standard_ui_vertices.len()) as u64).unwrap(),
        );

        Self {
            gfx_ctx,
            depth_stencil,
            bind_group_layout_cache,
            pipeline_layout_cache,
            pipeline_cache,
            frame_buffer_allocator,
            standard_ui_vertex_buffer,
        }
    }

    pub fn bind_group_layout_cache(&mut self) -> &mut BindGroupLayoutCache {
        &mut self.bind_group_layout_cache
    }

    pub fn pipeline_layout_cache(&mut self) -> &mut PipelineLayoutCache {
        &mut self.pipeline_layout_cache
    }

    pub fn pipeline_cache(&mut self) -> &mut PipelineCache {
        &mut self.pipeline_cache
    }

    pub fn standard_ui_vertex_buffer(&self) -> &GenericBufferAllocation<Buffer> {
        &self.standard_ui_vertex_buffer
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.depth_stencil.resize(size);
    }

    pub fn create_encoder(&self) -> CommandEncoder {
        self.gfx_ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None })
    }

    pub fn begin_frame_buffer_render_pass<'e>(
        &'e self,
        encoder: &'e mut CommandEncoder,
        surface_texture_view: &'e TextureView,
        clear_mode: &CameraClearMode,
    ) -> Result<RenderPass<'e>, SurfaceError> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &surface_texture_view,
                resolve_target: None,
                ops: Operations {
                    load: match clear_mode {
                        CameraClearMode::Keep => LoadOp::Load,
                        CameraClearMode::All { color, .. } => LoadOp::Clear(Color {
                            r: color.r as f64,
                            g: color.g as f64,
                            b: color.b as f64,
                            a: color.a as f64,
                        }),
                        CameraClearMode::DepthOnly { .. } => LoadOp::Load,
                    },
                    store: true,
                },
            })],
            depth_stencil_attachment: self.depth_stencil.texture_view().map(|view| {
                RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(Operations {
                        load: match clear_mode {
                            CameraClearMode::Keep => LoadOp::Load,
                            CameraClearMode::All { depth, .. } => LoadOp::Clear(*depth),
                            CameraClearMode::DepthOnly { depth, .. } => LoadOp::Clear(*depth),
                        },
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: match clear_mode {
                            CameraClearMode::Keep => LoadOp::Load,
                            CameraClearMode::All { stencil, .. } => LoadOp::Clear(*stencil),
                            CameraClearMode::DepthOnly { stencil, .. } => LoadOp::Clear(*stencil),
                        },
                        store: true,
                    }),
                }
            }),
        });
        Ok(render_pass)
    }

    /// Constructs a rendering command for the given object by encoding per-instance data into a buffer.
    pub fn build_rendering_command<'r>(
        &mut self,
        object_id: ObjectId,
        object_hierarchy: &ObjectHierarchy,
        renderer: &'r dyn Renderer,
    ) -> RenderingCommand<'r> {
        build_rendering_command(
            object_id,
            object_hierarchy,
            renderer,
            &mut self.frame_buffer_allocator,
        )
    }

    pub fn finish_frame(&mut self, command_buffers: Vec<CommandBuffer>) {
        self.gfx_ctx
            .queue
            .submit(vec![self.frame_buffer_allocator.finish()]);
        self.gfx_ctx.queue.submit(command_buffers);
    }
}
