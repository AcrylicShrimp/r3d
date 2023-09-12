use crate::{
    gfx::{Camera, MeshRenderer, Renderer, UIElementRenderer},
    math::Vec2,
    object::Object,
    ui::UISize,
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
        WriteStorage<'a, UIElementRenderer>,
        ReadStorage<'a, UISize>,
    );

    fn run(
        &mut self,
        (objects, cameras, mut mesh_renderers, mut ui_element_renderers, ui_sizes): Self::SystemData,
    ) {
        let context = use_context();
        let mut render_mgr = context.render_mgr_mut();
        let shader_mgr = context.shader_mgr();
        let world_mgr = context.object_mgr();
        let object_hierarchy = world_mgr.object_hierarchy();

        let surface_texture = context.gfx_ctx().surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = render_mgr.create_encoder();

        let mut camera_objects = (&objects, &cameras).join().collect::<Vec<_>>();
        camera_objects.sort_unstable_by_key(|&(_, camera)| camera.depth);

        for (object, camera) in camera_objects {
            if !object_hierarchy.is_active(object.object_id()) {
                continue;
            }

            let mut renderers = Vec::with_capacity(1024);
            let mut mesh_renderer_providers = Vec::with_capacity(1024);
            let mut ui_element_renderer_providers = Vec::with_capacity(1024);

            for (object, mesh_renderer) in (&objects, &mut mesh_renderers).join() {
                let object_id = object.object_id();

                if !object_hierarchy.is_active(object.object_id()) {
                    continue;
                }

                if mesh_renderer.mask() & camera.mask == 0 {
                    continue;
                }

                let provider_index = mesh_renderer_providers.len();
                let bind_group_provider = mesh_renderer.bind_group_provider();
                let per_instance_data_provider = mesh_renderer.per_instance_data_provider();
                mesh_renderer_providers.push((bind_group_provider, per_instance_data_provider));

                renderers.push((
                    object_id,
                    mesh_renderer as &mut dyn Renderer,
                    ProviderIndex::MeshRenderer(provider_index),
                ));
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

                let provider_index = ui_element_renderer_providers.len();
                let bind_group_provider = ui_element_renderer.bind_group_provider();
                let per_instance_data_provider = ui_element_renderer
                    .per_instance_data_provider(Vec2::new(ui_size.width, ui_size.height));
                ui_element_renderer_providers
                    .push((bind_group_provider, per_instance_data_provider));

                renderers.push((
                    object_id,
                    ui_element_renderer as &mut dyn Renderer,
                    ProviderIndex::UIElementRenderer(provider_index),
                ));
            }

            let mut commands = Vec::with_capacity(renderers.len());

            for (object_id, renderer, provider_index) in renderers {
                let (bind_group_provider, per_instance_data_provider) = match provider_index {
                    ProviderIndex::MeshRenderer(index) => {
                        let (bind_group_provider, per_instance_data_provider) =
                            &mesh_renderer_providers[index];
                        (bind_group_provider as _, per_instance_data_provider as _)
                    }
                    ProviderIndex::UIElementRenderer(index) => {
                        let (bind_group_provider, per_instance_data_provider) =
                            &ui_element_renderer_providers[index];
                        (bind_group_provider as _, per_instance_data_provider as _)
                    }
                };

                if let Some(cmd) = render_mgr.build_rendering_command(
                    camera.bind_group.clone(),
                    object_id,
                    object_hierarchy,
                    renderer,
                    bind_group_provider,
                    per_instance_data_provider,
                    shader_mgr,
                ) {
                    commands.push(cmd);
                }
            }

            let mut render_pass = render_mgr
                .begin_frame_buffer_render_pass(
                    &mut encoder,
                    &surface_texture_view,
                    &camera.clear_mode,
                )
                .unwrap();

            for cmd in &commands {
                cmd.render(&mut render_pass);
            }
        }

        render_mgr.finish_frame(vec![encoder.finish()]);
        surface_texture.present();
    }
}

enum ProviderIndex {
    MeshRenderer(usize),
    UIElementRenderer(usize),
}
