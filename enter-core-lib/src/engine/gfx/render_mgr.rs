use super::{
    build_rendering_command, BindGroupLayoutCache, DepthStencil, DepthStencilMode,
    FrameBufferAllocator, GfxContextHandle, PipelineCache, PipelineLayoutCache, Renderer,
    RenderingCommand, ShaderManager,
};
use crate::engine::object::{ObjectHierarchy, ObjectId};
use wgpu::{
    Color, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, SurfaceError, TextureView,
};
use winit::dpi::PhysicalSize;

pub struct RenderManager {
    gfx_ctx: GfxContextHandle,
    depth_stencil: DepthStencil,
    bind_group_layout_cache: BindGroupLayoutCache,
    pipeline_layout_cache: PipelineLayoutCache,
    pipeline_cache: PipelineCache,
    frame_buffer_allocator: FrameBufferAllocator,
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

        Self {
            gfx_ctx,
            depth_stencil,
            bind_group_layout_cache,
            pipeline_layout_cache,
            pipeline_cache,
            frame_buffer_allocator,
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
        clear_color: Color,
        clear_depth: f32,
        clear_stencil: u32,
    ) -> Result<RenderPass<'e>, SurfaceError> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &surface_texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: self.depth_stencil.texture_view().map(|view| {
                RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(clear_depth),
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: LoadOp::Clear(clear_stencil),
                        store: true,
                    }),
                }
            }),
        });
        Ok(render_pass)
    }

    pub fn build_rendering_command(
        &mut self,
        object_id: ObjectId,
        object_hierarchy: &ObjectHierarchy,
        renderer: &mut dyn Renderer,
        shader_mgr: &ShaderManager,
    ) -> Option<RenderingCommand> {
        build_rendering_command(
            object_id,
            object_hierarchy,
            renderer,
            shader_mgr,
            &mut self.pipeline_cache,
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
