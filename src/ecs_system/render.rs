use crate::{
    gfx::{Camera, MeshRenderer},
    object::Object,
    use_context,
};
use specs::prelude::*;

pub struct RenderSystem;

impl RenderSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, Object>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, MeshRenderer>,
    );

    fn run(&mut self, (objects, cameras, mut mesh_renderers): Self::SystemData) {
        let context = use_context();
        let mut render_mgr = context.render_mgr_mut();
        let shader_mgr = context.shader_mgr();
        let world_mgr = context.object_mgr();
        let object_hierarchy = world_mgr.object_hierarchy();

        let surface_texture = context.gfx_ctx().surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = render_mgr.create_encoder();

        for (object, camera) in (&objects, &cameras).join() {
            if !object_hierarchy.is_active(object.object_id()) {
                continue;
            }

            let mesh_renderers = (&objects, &mut mesh_renderers).join().collect::<Vec<_>>();
            let mut rendering_commands = Vec::new();

            for (object, mesh_renderer) in mesh_renderers {
                let object_id = object.object_id();

                if !object_hierarchy.is_active(object.object_id()) {
                    continue;
                }

                if mesh_renderer.mask() & camera.mask == 0 {
                    continue;
                }

                if let Some(command) = render_mgr.build_rendering_command(
                    camera.bind_group.clone(),
                    object_id,
                    object_hierarchy,
                    mesh_renderer,
                    shader_mgr,
                ) {
                    rendering_commands.push(command);
                }
            }

            let mut render_pass = render_mgr
                .begin_frame_buffer_render_pass(
                    &mut encoder,
                    &surface_texture_view,
                    &camera.clear_mode,
                )
                .unwrap();

            for rendering_command in &rendering_commands {
                rendering_command.render(&mut render_pass);
            }
        }

        render_mgr.finish_frame(vec![encoder.finish()]);
        surface_texture.present();
    }
}
