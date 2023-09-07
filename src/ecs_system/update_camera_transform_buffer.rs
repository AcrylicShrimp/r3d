use crate::{gfx::Camera, object::Object, ContextHandle};
use specs::prelude::*;

pub struct UpdateCameraTransformBufferSystem {
    ctx: ContextHandle,
}

impl UpdateCameraTransformBufferSystem {
    pub fn new(ctx: ContextHandle) -> Self {
        Self { ctx }
    }
}

impl<'a> System<'a> for UpdateCameraTransformBufferSystem {
    type SystemData = (ReadStorage<'a, Object>, ReadStorage<'a, Camera>);

    fn run(&mut self, (objects, cameras): Self::SystemData) {
        let world_mgr = self.ctx.world_mgr();
        let screen_mgr = self.ctx.screen_mgr();
        let object_hierarchy = world_mgr.object_hierarchy();

        for (object, camera) in (&objects, &cameras).join() {
            if !object_hierarchy.is_active(object.object_id()) {
                continue;
            }

            let object_id = object.object_id();
            let matrix = object_hierarchy.matrix(object_id);

            camera.update_buffer(&screen_mgr, &self.ctx.gfx_ctx.queue, matrix);
        }
    }
}
