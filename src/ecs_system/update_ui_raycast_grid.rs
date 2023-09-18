use crate::{object::Object, ui::UIElement, ContextHandle};
use specs::prelude::*;

pub struct UpdateUIRaycastGrid {
    ctx: ContextHandle,
}

impl UpdateUIRaycastGrid {
    pub fn new(ctx: ContextHandle) -> Self {
        Self { ctx }
    }
}

impl<'a> System<'a> for UpdateUIRaycastGrid {
    type SystemData = (ReadStorage<'a, Object>, ReadStorage<'a, UIElement>);

    fn run(&mut self, (objects, ui_elements): Self::SystemData) {
        let mut ui_raycast_mgr = self.ctx.ui_raycast_mgr_mut();

        let object_mgr = self.ctx.object_mgr();
        let hierarchy = object_mgr.object_hierarchy();

        for (object, _) in (&objects, &ui_elements).join() {
            if hierarchy.is_dirty(object.object_id()) {
                ui_raycast_mgr.add_object(object_mgr.object_handle(object.object_id()));
            }
        }
    }
}
