use crate::{
    gfx::{
        BindGroupLayoutCache, Camera, MeshRenderer, Renderer, UIElementRenderer, UITextRenderer,
    },
    object::Object,
    ui::UISize,
    use_context,
};
use image::EncodableLayout;
use specs::prelude::*;
use std::mem::size_of;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Device, ShaderStages,
};

pub struct RenderSystem {
    screen_size_buffer: Buffer,
    screen_size_bind_group: BindGroup,
}

impl RenderSystem {
    pub fn new(device: &Device, bind_group_layout_cache: &mut BindGroupLayoutCache) -> Self {
        let screen_size_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<[f32; 4]>() as u64 as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let screen_size_bind_group_layout =
            bind_group_layout_cache.create_layout(vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(BufferSize::new(size_of::<[f32; 4]>() as u64).unwrap()),
                },
                count: None,
            }]);
        let screen_size_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: screen_size_bind_group_layout.as_ref(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        Self {
            screen_size_buffer,
            screen_size_bind_group,
        }
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, Object>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, MeshRenderer>,
        WriteStorage<'a, UIElementRenderer>,
        WriteStorage<'a, UITextRenderer>,
        ReadStorage<'a, UISize>,
    );

    fn run(
        &mut self,
        (
            objects,
            cameras,
            mut mesh_renderers,
            mut ui_element_renderers,
            mut ui_text_renderers,
            ui_sizes,
        ): Self::SystemData,
    ) {
        let context = use_context();
        let mut render_mgr = context.render_mgr_mut();
        let shader_mgr = context.shader_mgr();
        let world_mgr = context.object_mgr();
        let object_hierarchy = world_mgr.object_hierarchy();

        context
            .gfx_ctx()
            .queue
            .write_buffer(&self.screen_size_buffer, 0, {
                let screen_mgr = context.screen_mgr();
                [
                    screen_mgr.width() as f32,
                    screen_mgr.height() as f32,
                    0.0f32,
                    0.0f32,
                ]
                .as_bytes()
            });

        let surface_texture = context.gfx_ctx().surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = render_mgr.create_encoder();

        let mut camera_objects = (&objects, &cameras).join().collect::<Vec<_>>();
        camera_objects.sort_unstable_by_key(|&(_, camera)| camera.depth);

        for (object, camera) in camera_objects {
            let pipeline_cache = render_mgr.pipeline_cache();

            if !object_hierarchy.is_active(object.object_id()) {
                continue;
            }

            let mut mesh_sub_renderers = Vec::with_capacity(1024);

            let mut ui_element_sub_renderers = Vec::with_capacity(1024);
            let mut ui_text_sub_renderers = Vec::with_capacity(1024);

            for (object, mesh_renderer) in (&objects, &mut mesh_renderers).join() {
                let object_id = object.object_id();

                if !object_hierarchy.is_active(object.object_id()) {
                    continue;
                }

                if mesh_renderer.mask() & camera.mask == 0 {
                    continue;
                }

                let renderer = if let Some(renderer) =
                    mesh_renderer.sub_renderer(shader_mgr, pipeline_cache)
                {
                    renderer
                } else {
                    continue;
                };

                mesh_sub_renderers.push((object_id, renderer));
            }

            for (object, ui_element_renderer, ui_size) in
                (&objects, &mut ui_element_renderers, &ui_sizes).join()
            {
                let object_id = object.object_id();

                if !object_hierarchy.is_active(object.object_id()) {
                    continue;
                }

                if ui_element_renderer.mask() & camera.mask == 0 {
                    continue;
                }

                let renderer = if let Some(renderer) =
                    ui_element_renderer.sub_renderer(*ui_size, shader_mgr, pipeline_cache)
                {
                    renderer
                } else {
                    continue;
                };

                ui_element_sub_renderers.push((
                    object_hierarchy.index(object_id),
                    object_id,
                    renderer,
                ));
            }

            for (object, ui_text_renderer) in (&objects, &mut ui_text_renderers).join() {
                let object_id = object.object_id();

                if !object_hierarchy.is_active(object.object_id()) {
                    continue;
                }

                if ui_text_renderer.mask() & camera.mask == 0 {
                    continue;
                }

                let renderers = if let Some(renderers) =
                    ui_text_renderer.sub_renderers(shader_mgr, pipeline_cache)
                {
                    renderers
                } else {
                    continue;
                };

                for renderer in renderers {
                    ui_text_sub_renderers.push((
                        object_hierarchy.index(object_id),
                        object_id,
                        renderer,
                    ));
                }
            }

            let mut ui_sub_renderers =
                Vec::with_capacity(ui_element_sub_renderers.len() + ui_text_sub_renderers.len());

            for (index, object_id, renderer) in &ui_element_sub_renderers {
                ui_sub_renderers.push((*index, *object_id, renderer as &dyn Renderer));
            }

            for (index, object_id, renderer) in &ui_text_sub_renderers {
                ui_sub_renderers.push((*index, *object_id, renderer as &dyn Renderer));
            }

            ui_sub_renderers.sort_unstable_by_key(|&(index, _, _)| index);

            let mut commands =
                Vec::with_capacity(mesh_sub_renderers.len() + ui_sub_renderers.len());

            for (object_id, renderer) in &mesh_sub_renderers {
                let command =
                    render_mgr.build_rendering_command(*object_id, object_hierarchy, renderer);
                commands.push(command);
            }

            for (_, object_id, renderer) in &ui_sub_renderers {
                let command =
                    render_mgr.build_rendering_command(*object_id, object_hierarchy, *renderer);
                commands.push(command);
            }

            let mut render_pass = render_mgr
                .begin_frame_buffer_render_pass(
                    &mut encoder,
                    &surface_texture_view,
                    &camera.clear_mode,
                )
                .unwrap();

            for cmd in &commands {
                cmd.render(
                    &mut render_pass,
                    &camera.bind_group,
                    &self.screen_size_bind_group,
                );
            }
        }

        render_mgr.finish_frame(vec![encoder.finish()]);
        surface_texture.present();
    }
}
