use crate::engine::{gfx::MeshRenderer, use_context};
use specs::prelude::*;
use wgpu::Color;

pub struct RenderSystem;

impl RenderSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (WriteStorage<'a, MeshRenderer>,);

    fn run(&mut self, (mut mesh_renderers,): Self::SystemData) {
        let context = use_context();
        let mut render_mgr = context.render_mgr_mut();
        let shader_mgr = context.shader_mgr();

        let surface_texture = context.gfx_ctx().surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = render_mgr.create_encoder();

        let mesh_renderers = (&mut mesh_renderers,).join().collect::<Vec<_>>();

        let mut rendering_commands = Vec::new();

        for (mesh_renderer,) in mesh_renderers {
            if let Some(command) = render_mgr.build_rendering_command(mesh_renderer, shader_mgr) {
                rendering_commands.push(command);
            }
        }

        {
            let mut render_pass = render_mgr
                .begin_frame_buffer_render_pass(
                    &mut encoder,
                    &surface_texture_view,
                    Color::GREEN,
                    1.0,
                    0,
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
